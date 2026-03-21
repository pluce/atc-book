use base64::Engine;
use image::ImageEncoder;
use pdfium_render::prelude::*;
use std::sync::OnceLock;
use std::time::Instant;

/// Render width in pixels (2x base for crisp zoom at 200%)
const RENDER_WIDTH: i32 = 1200;
/// Maximum render height in pixels
const RENDER_MAX_HEIGHT: i32 = 1900;
/// Max number of rendered PNG pages to keep on disk
const MAX_RENDERED_PAGES: usize = 200;

/// A rendered PDF page as a base64 PNG data URL
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedPage {
    /// `data:image/png;base64,...`
    pub data_url: String,
    /// 0-based page index
    pub index: usize,
}

/// Download a PDF from a URL and render all pages as PNG data URLs.
/// Uses the local file cache to avoid re-downloading.
#[allow(dead_code)]
pub async fn fetch_and_render(url: &str) -> Result<Vec<RenderedPage>, String> {
    let started = Instant::now();
    // Try rendered PNG cache first
    if let Some(pages) = load_rendered_cache(url)? {
        println!(
            "[pdf] render cache hit url={} pages={} elapsed_ms={}",
            url,
            pages.len(),
            started.elapsed().as_millis()
        );
        return Ok(pages);
    }

    let bytes = fetch_pdf_bytes(url).await?;

    let (tx, rx) = async_channel::bounded::<Result<Vec<RenderedPage>, String>>(1);
    let url_for_render = url.to_string();
    std::thread::spawn(move || {
        let _ = tx.send_blocking(render_pdf_bytes_with_cache(&url_for_render, &bytes));
    });
    let pages = rx.recv().await.map_err(|e| format!("Channel: {e}"))??;
    println!(
        "[pdf] render done url={} pages={} elapsed_ms={}",
        url,
        pages.len(),
        started.elapsed().as_millis()
    );
    Ok(pages)
}

/// Pre-render a PDF into the rendered PNG cache.
pub async fn pre_render_pdf(url: &str) -> Result<(), String> {
    if load_rendered_cache(url)?.is_some() {
        return Ok(());
    }
    let bytes = fetch_pdf_bytes(url).await?;
    let (tx, rx) = async_channel::bounded::<Result<(), String>>(1);
    let url = url.to_string();
    std::thread::spawn(move || {
        let result = render_pdf_bytes_with_cache(&url, &bytes).map(|_| ());
        let _ = tx.send_blocking(result);
    });
    rx.recv().await.map_err(|e| format!("Channel: {e}"))?
}

/// Download and render multiple PDFs as a single ordered document.
/// URLs are rendered in order and concatenated: main chart first, then linked docs.
pub async fn fetch_and_render_many(urls: &[String]) -> Result<Vec<RenderedPage>, String> {
    if urls.is_empty() {
        return Err("No PDF URL provided".to_string());
    }

    println!("[pdf] aggregate render start docs={} urls={:?}", urls.len(), urls);

    // Keep one slot per URL to preserve final ordering.
    let mut by_doc: Vec<Option<Vec<RenderedPage>>> = vec![None; urls.len()];
    let mut to_render: Vec<(usize, String, Vec<u8>)> = Vec::new();

    for (doc_idx, url) in urls.iter().enumerate() {
        if let Some(pages) = load_rendered_cache(url)? {
            println!(
                "[pdf] aggregate render doc={} url={} pages={} cache=rendered",
                doc_idx,
                url,
                pages.len()
            );
            by_doc[doc_idx] = Some(pages);
            continue;
        }

        let bytes = fetch_pdf_bytes(url).await?;
        to_render.push((doc_idx, url.clone(), bytes));
    }

    if !to_render.is_empty() {
        let (tx, rx) = async_channel::bounded::<Result<Vec<(usize, Vec<RenderedPage>)>, String>>(1);
        std::thread::spawn(move || {
            let mut out = Vec::with_capacity(to_render.len());
            for (doc_idx, url, bytes) in to_render {
                let started = Instant::now();
                match render_pdf_bytes_with_cache(&url, &bytes) {
                    Ok(pages) => {
                        println!(
                            "[pdf] render done url={} pages={} elapsed_ms={}",
                            url,
                            pages.len(),
                            started.elapsed().as_millis()
                        );
                        out.push((doc_idx, pages));
                    }
                    Err(e) => {
                        let _ = tx.send_blocking(Err(e));
                        return;
                    }
                }
            }
            let _ = tx.send_blocking(Ok(out));
        });

        for (doc_idx, pages) in rx.recv().await.map_err(|e| format!("Channel: {e}"))?? {
            by_doc[doc_idx] = Some(pages);
        }
    }

    let mut merged = Vec::new();
    for (doc_idx, url) in urls.iter().enumerate() {
        let mut pages = by_doc[doc_idx]
            .take()
            .ok_or_else(|| format!("Missing rendered pages for doc {doc_idx}: {url}"))?;
        println!(
            "[pdf] aggregate render doc={} url={} pages={}",
            doc_idx,
            url,
            pages.len()
        );
        merged.append(&mut pages);
    }

    for (index, page) in merged.iter_mut().enumerate() {
        page.index = index;
    }

    println!("[pdf] aggregate render done total_pages={}", merged.len());

    Ok(merged)
}

/// Download and render only the first page of each PDF, preserving document order.
/// Useful to display content quickly while the full render continues in background.
#[allow(dead_code)]
pub async fn fetch_and_render_many_first_pages(urls: &[String]) -> Result<Vec<RenderedPage>, String> {
    if urls.is_empty() {
        return Err("No PDF URL provided".to_string());
    }

    println!(
        "[pdf] aggregate first-page render start docs={} urls={:?}",
        urls.len(),
        urls
    );

    let mut out = Vec::with_capacity(urls.len());
    for (doc_idx, url) in urls.iter().enumerate() {
        if let Some(mut pages) = load_rendered_cache(url)? {
            if let Some(mut first) = pages.drain(..).next() {
                first.index = out.len();
                println!(
                    "[pdf] aggregate first-page doc={} url={} cache=rendered",
                    doc_idx, url
                );
                out.push(first);
                continue;
            }
        }

        let bytes = fetch_pdf_bytes(url).await?;
        let (tx, rx) = async_channel::bounded::<Result<RenderedPage, String>>(1);
        let url_for_render = url.clone();
        std::thread::spawn(move || {
            let _ = tx.send_blocking(render_first_page_with_cache(&url_for_render, &bytes));
        });

        let mut first = rx.recv().await.map_err(|e| format!("Channel: {e}"))??;
        first.index = out.len();
        println!(
            "[pdf] aggregate first-page doc={} url={} rendered=1",
            doc_idx, url
        );
        out.push(first);
    }

    println!(
        "[pdf] aggregate first-page render done pages={}",
        out.len()
    );
    Ok(out)
}

/// Download and render only the first page of a single PDF.
/// Useful for progressive display: show content quickly before full render completes.
pub async fn fetch_and_render_first_page(url: &str) -> Result<RenderedPage, String> {
    let started = Instant::now();

    if let Some(mut pages) = load_rendered_cache(url)? {
        if let Some(mut first) = pages.drain(..).next() {
            first.index = 0;
            println!(
                "[pdf] first-page cache hit url={} elapsed_ms={}",
                url,
                started.elapsed().as_millis()
            );
            return Ok(first);
        }
    }

    let bytes = fetch_pdf_bytes(url).await?;
    let (tx, rx) = async_channel::bounded::<Result<RenderedPage, String>>(1);
    let url_for_render = url.to_string();
    std::thread::spawn(move || {
        let _ = tx.send_blocking(render_first_page_with_cache(&url_for_render, &bytes));
    });

    let page = rx.recv().await.map_err(|e| format!("Channel: {e}"))??;
    println!(
        "[pdf] first-page rendered url={} elapsed_ms={}",
        url,
        started.elapsed().as_millis()
    );
    Ok(page)
}

/// Pre-render multiple PDFs in order.
pub async fn pre_render_pdf_many(urls: &[String]) -> Result<(), String> {
    if urls.is_empty() {
        return Ok(());
    }

    println!("[pdf] aggregate prerender start docs={} urls={:?}", urls.len(), urls);

    for (doc_idx, url) in urls.iter().enumerate() {
        println!("[pdf] aggregate prerender doc={} url={}", doc_idx, url);
        pre_render_pdf(url).await?;
    }
    println!("[pdf] aggregate prerender done docs={}", urls.len());
    Ok(())
}

/// Preload a PDF into local cache for offline use, without rendering pages.
#[allow(dead_code)]
pub async fn prefetch_pdf(url: &str) -> Result<(), String> {
    let _ = fetch_pdf_bytes(url).await?;
    Ok(())
}

/// Fetch PDF bytes, checking the local disk cache first.
async fn fetch_pdf_bytes(url: &str) -> Result<Vec<u8>, String> {
    let started = Instant::now();
    // 1. Try local cache
    let cached_path = {
        let conn = crate::persistence::db().lock().unwrap();
        crate::persistence::cache::get_pdf_path(&conn, url)
    };
    if let Some(path) = cached_path {
        if let Ok(bytes) = std::fs::read(&path) {
            println!(
                "[pdf] bytes cache hit url={} size={} elapsed_ms={}",
                url,
                bytes.len(),
                started.elapsed().as_millis()
            );
            return Ok(bytes);
        }
    }

    // 2. Download
    let bytes = http_client()
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Réseau: {e}"))?
        .error_for_status()
        .map_err(|e| format!("HTTP: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("Lecture: {e}"))?;

    // 3. Save to disk cache
    let cache_dir = crate::persistence::pdf_cache_dir();
    // Use a hash of the URL as filename to avoid path issues
    let hash = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        url.hash(&mut hasher);
        hasher.finish()
    };
    let ext = if url.ends_with(".pdf") { "pdf" } else { "bin" };
    let local_path = cache_dir.join(format!("{hash:016x}.{ext}"));
    if let Ok(()) = std::fs::write(&local_path, &bytes) {
        let conn = crate::persistence::db().lock().unwrap();
        crate::persistence::cache::put_pdf_entry(
            &conn,
            url,
            &local_path.to_string_lossy(),
            bytes.len() as u64,
        );
    }

    println!(
        "[pdf] bytes downloaded url={} size={} elapsed_ms={}",
        url,
        bytes.len(),
        started.elapsed().as_millis()
    );

    Ok(bytes.to_vec())
}

fn load_rendered_cache(url: &str) -> Result<Option<Vec<RenderedPage>>, String> {
    let cached = {
        let conn = crate::persistence::db().lock().unwrap();
        crate::persistence::cache::get_rendered_pages(&conn, url)
    };
    if let Some(paths) = cached {
        let mut pages = Vec::with_capacity(paths.len());
        for (idx, path) in paths {
            let bytes = std::fs::read(&path).map_err(|e| format!("Lecture PNG: {e}"))?;
            let data_url = png_bytes_to_data_url(&bytes)?;
            pages.push(RenderedPage {
                data_url,
                index: idx,
            });
        }
        return Ok(Some(pages));
    }
    Ok(None)
}

/// Render all pages of a PDF from raw bytes.
/// This is a blocking operation — call from a dedicated thread.
#[allow(dead_code)]
pub(crate) fn render_pdf_bytes(pdf_bytes: &[u8]) -> Result<Vec<RenderedPage>, String> {
    let pdfium = pdfium_auto::bind_bundled().map_err(|e| format!("PDFium init: {e}"))?;

    let document = pdfium
        .load_pdf_from_byte_vec(pdf_bytes.to_vec(), None)
        .map_err(|e| format!("PDF ouverture: {e}"))?;

    let page_count = document.pages().len();
    let mut pages = Vec::with_capacity(page_count as usize);

    for i in 0..page_count {
        pages.push(render_page(&document, i)?);
    }

    Ok(pages)
}

fn render_pdf_bytes_with_cache(url: &str, pdf_bytes: &[u8]) -> Result<Vec<RenderedPage>, String> {
    let pdfium = pdfium_auto::bind_bundled().map_err(|e| format!("PDFium init: {e}"))?;

    render_pdf_bytes_with_cache_for_pdfium(&pdfium, url, pdf_bytes)
}

fn render_first_page_with_cache(url: &str, pdf_bytes: &[u8]) -> Result<RenderedPage, String> {
    let pdfium = pdfium_auto::bind_bundled().map_err(|e| format!("PDFium init: {e}"))?;

    let document = pdfium
        .load_pdf_from_byte_vec(pdf_bytes.to_vec(), None)
        .map_err(|e| format!("PDF ouverture: {e}"))?;

    let png_bytes = render_page_png(&document, 0)?;
    let data_url = png_bytes_to_data_url(&png_bytes)?;

    let cache_dir = crate::persistence::rendered_cache_dir();
    let hash = url_hash(url);
    let local_path = cache_dir.join(format!("{hash:016x}_0.png"));
    if std::fs::write(&local_path, &png_bytes).is_ok() {
        let conn = crate::persistence::db().lock().unwrap();
        crate::persistence::cache::put_rendered_page(&conn, url, 0, &local_path.to_string_lossy());
    }

    Ok(RenderedPage {
        data_url,
        index: 0,
    })
}

fn render_pdf_bytes_with_cache_for_pdfium(
    pdfium: &Pdfium,
    url: &str,
    pdf_bytes: &[u8],
) -> Result<Vec<RenderedPage>, String> {

    let document = pdfium
        .load_pdf_from_byte_vec(pdf_bytes.to_vec(), None)
        .map_err(|e| format!("PDF ouverture: {e}"))?;

    let page_count = document.pages().len();
    let mut pages = Vec::with_capacity(page_count as usize);

    let cache_dir = crate::persistence::rendered_cache_dir();
    let hash = url_hash(url);
    for i in 0..page_count {
        let png_bytes = render_page_png(&document, i)?;
        let data_url = png_bytes_to_data_url(&png_bytes)?;
        let local_path = cache_dir.join(format!("{hash:016x}_{i}.png"));
        if std::fs::write(&local_path, &png_bytes).is_ok() {
            let conn = crate::persistence::db().lock().unwrap();
            crate::persistence::cache::put_rendered_page(
                &conn,
                url,
                i as usize,
                &local_path.to_string_lossy(),
            );
        }
        pages.push(RenderedPage {
            data_url,
            index: i as usize,
        });
    }

    {
        let conn = crate::persistence::db().lock().unwrap();
        crate::persistence::cache::prune_rendered_cache(&conn, MAX_RENDERED_PAGES);
    }

    Ok(pages)
}

/// Render a single page to a PNG data URL.
#[allow(dead_code)]
fn render_page(document: &PdfDocument, index: u16) -> Result<RenderedPage, String> {
    let png_bytes = render_page_png(document, index)?;
    let data_url = png_bytes_to_data_url(&png_bytes)?;
    Ok(RenderedPage {
        data_url,
        index: index as usize,
    })
}

fn render_page_png(document: &PdfDocument, index: u16) -> Result<Vec<u8>, String> {
    let page = document
        .pages()
        .get(index)
        .map_err(|e| format!("Page {index}: {e}"))?;

    let config = PdfRenderConfig::new()
        .set_target_width(RENDER_WIDTH)
        .set_maximum_height(RENDER_MAX_HEIGHT);

    let bitmap = page
        .render_with_config(&config)
        .map_err(|e| format!("Rendu page {index}: {e}"))?;

    bitmap_to_png_bytes(&bitmap, index)
}

/// Convert a bitmap to a PNG base64 data URL.
#[allow(dead_code)]
fn bitmap_to_data_url(bitmap: &PdfBitmap, page_index: u16) -> Result<String, String> {
    let png = bitmap_to_png_bytes(bitmap, page_index)?;
    png_bytes_to_data_url(&png)
}

fn bitmap_to_png_bytes(bitmap: &PdfBitmap, page_index: u16) -> Result<Vec<u8>, String> {
    let width = bitmap.width() as usize;
    let height = bitmap.height() as usize;
    let rgba = bitmap.as_raw_bytes().to_vec();

    let mut png_buf = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png_buf)
        .write_image(
            &rgba,
            width as u32,
            height as u32,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| format!("PNG encode page {page_index}: {e}"))?;

    Ok(png_buf)
}

fn png_bytes_to_data_url(bytes: &[u8]) -> Result<String, String> {
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:image/png;base64,{b64}"))
}

fn url_hash(url: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut hasher);
    hasher.finish()
}

fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .pool_max_idle_per_host(8)
            .tcp_keepalive(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

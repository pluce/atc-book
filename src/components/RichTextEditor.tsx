'use client';

import { useEffect, useRef } from 'react';
import Quill from 'quill';
import 'quill/dist/quill.snow.css';

interface RichTextEditorProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
}

export default function RichTextEditor({ value, onChange, placeholder, className }: RichTextEditorProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const quillRef = useRef<Quill | null>(null);

  useEffect(() => {
    if (containerRef.current) {
        // Defensive cleanup: Ensure container is empty before initializing
        containerRef.current.innerHTML = '';
        
        // Create an element for Quill to mount on
        const editorDiv = document.createElement('div');
        containerRef.current.appendChild(editorDiv);

        const q = new Quill(editorDiv, {
            theme: 'snow',
            placeholder: placeholder,
            modules: {
                toolbar: [
                    [{ 'header': [1, 2, 3, false] }],
                    ['bold', 'italic', 'underline', 'strike'],
                    [{ 'list': 'ordered'}, { 'list': 'bullet' }],
                    [{ 'color': [] }, { 'background': [] }],
                    ['clean']
                ]
            }
        });
        
        quillRef.current = q;

        // Set initial value
        if (value) {
            // Use local clipboard API to avoid triggering text-change with user source (though dangerousPasteHtml usually is API source)
            q.clipboard.dangerouslyPasteHTML(value);
        }

        // Handle changes
        q.on('text-change', (delta, oldDelta, source) => {
             if (source === 'user') {
                 const html = q.root.innerHTML;
                 // Avoid triggering updates for empty default paragraph
                 const content = html === '<p><br></p>' ? '' : html;
                 onChange(content);
             }
        });
    }
    
    // Cleanup function
    return () => {
        if (containerRef.current) {
            containerRef.current.innerHTML = '';
        }
        quillRef.current = null;
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); 

  // Sync value from props (External updates)
  useEffect(() => {
      if (quillRef.current && value !== undefined) {
          const q = quillRef.current;
          const currentContent = q.root.innerHTML === '<p><br></p>' ? '' : q.root.innerHTML;
          
          // Only update if content is substantially different to prevent loop/cursor jumps
          // This comparison assumes HTML stability. 
          if (value !== currentContent) {
               // This is an external change (e.g. Loading a save)
               // We don't try to preserve cursor here as standard behavior for "Controlled" input 
               // that gets completely replaced is to reset.
               // However, for typing loops (Parent receives A, sends A back), the === check prevents update.
               q.clipboard.dangerouslyPasteHTML(value);
          }
      }
  }, [value]);

  return (
    <div className={`quill ${className || ''}`} ref={containerRef}>
        {/* Quill will append .ql-toolbar and .ql-container here */}
    </div>
  );
}

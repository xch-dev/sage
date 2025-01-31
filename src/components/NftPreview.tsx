import { NftData } from '@/bindings';
import { isImage, isVideo, isText, isJson, nftUri } from '@/lib/nftUri';
import { cn } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useEffect, useRef } from 'react';

interface NftPreviewProps {
  data: NftData | null;
  compact?: boolean;
  className?: string;
  name?: string | null;
}

// Count emoji sequences as single characters
const getVisualLength = (str: string) => {
  return str.replace(
    /\p{Emoji_Modifier_Base}\p{Emoji_Modifier}|\p{Emoji}(\u200d\p{Emoji})*|\p{Emoji}/gu,
    '_',
  ).length;
};

export function NftPreview({
  data,
  compact = false,
  className,
  name,
}: NftPreviewProps) {
  const textRef = useRef<HTMLPreElement>(null);
  const uri = nftUri(data?.mime_type ?? null, data?.blob ?? null);

  // Dynamic text sizing effect for plain text and JSON
  useEffect(() => {
    const el = textRef.current;
    if (el && (isText(data?.mime_type) || isJson(data?.mime_type))) {
      const { width, height } = el.getBoundingClientRect();

      // Get content dimensions
      const content = el.textContent || '';
      const lines = content.split('\n');
      const maxLineLength = Math.max(
        ...lines.map((line) => getVisualLength(line)),
      );
      const lineCount = lines.length;

      // Container dimensions in character units (assuming 16px per char)
      const containerColumns = width / 16;
      const containerRows = height / 16;

      // Calculate scaling factors based on content
      const contentColumns = maxLineLength;
      const contentRows = lineCount;

      // Scale based on both container and content
      const columnScale = containerColumns / contentColumns;
      const rowScale = containerRows / contentRows;

      // Base scale (from ord)
      const baseScale = compact ? 40 : 95;

      // For multi-line content, we should be more conservative with height scaling
      const heightAdjustment = lineCount > 1 ? 0.45 : 1; // Reduce scale for multi-line content

      // Apply the most constraining scale
      const scale = Math.min(
        baseScale,
        baseScale * columnScale,
        baseScale * rowScale * heightAdjustment, // Apply height adjustment here
      );

      el.style.fontSize = `min(${scale / containerColumns}vw, ${scale / containerRows}vh)`;
      el.style.opacity = '1';

      // Special case for very short content
      if (contentColumns <= 4 && contentRows <= 4) {
        const singleCharScale = compact ? 45 : 400; // Adjust these values
        el.style.fontSize = `min(${singleCharScale / containerColumns}vw, ${singleCharScale / containerRows}vh)`;
      }

      console.log('Content Analysis:', {
        type: isJson(data?.mime_type) ? 'JSON' : 'Text',
        compact,
        content,
        lineCount,
        maxLineLength,
        visualLength: getVisualLength(content),
        containerSize: { width, height },
        fontSize: el.style.fontSize,
        baseScale: baseScale,
        columnScale: columnScale,
        rowScale: rowScale,
        heightAdjustment,
        finalScale: scale,
      });
    }
  }, [data?.mime_type, compact]);

  if (isImage(data?.mime_type ?? null)) {
    return (
      <img
        alt={name ?? t`NFT artwork for unnamed NFT`}
        loading='lazy'
        width='150'
        height='150'
        className={cn(
          'h-auto w-auto object-cover transition-all aspect-square color-[transparent]',
          compact && 'group-hover:scale-105',
          className,
        )}
        src={uri}
      />
    );
  }

  if (isVideo(data?.mime_type ?? null)) {
    return (
      <video
        src={uri}
        controls
        className={cn(
          'h-auto w-auto object-cover transition-all aspect-square',
          compact && 'group-hover:scale-105',
          className,
        )}
      />
    );
  }

  if (isJson(data?.mime_type ?? null) || isText(data?.mime_type ?? null)) {
    const content = isJson(data?.mime_type ?? null)
      ? JSON.stringify(JSON.parse(uri), null, 2)
      : uri;

    return (
      <div
        className={cn(
          'grid h-full w-full place-items-center overflow-hidden',
          className,
        )}
        style={{
          gridTemplate: '1fr / 1fr',
          height: compact ? '150px' : '400px',
        }}
      >
        <pre
          ref={textRef}
          className={cn(
            'm-0 p-2 whitespace-pre-wrap break-words',
            isJson(data?.mime_type)
              ? 'font-mono text-left'
              : 'font-sans text-center',
            'bg-white border border-neutral-200 rounded-lg transition-all',
            compact && 'group-hover:scale-105',
          )}
          style={{
            gridColumn: '1 / 1',
            gridRow: '1 / 1',
            width: compact ? '150px' : '400px',
            opacity: 0, // Start invisible until sized
            maxHeight: compact ? '150px' : '80vh',
          }}
        >
          {content}
        </pre>
      </div>
    );
  }

  // Fallback for unsupported types
  return (
    <div
      className={cn(
        'flex items-center justify-center aspect-square bg-gray-100 text-gray-400',
        className,
      )}
    >
      <span className='text-sm'>{t`Unsupported content type`}</span>
    </div>
  );
}

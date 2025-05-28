import { platform } from '@tauri-apps/plugin-os';
import { shareText } from '@buildyourwebapp/tauri-plugin-sharesheet';
import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';

export async function exportText(text: string, title: string) {
  const isMobile = platform() === 'ios' || platform() === 'android';

  if (isMobile) {
    await shareText(text, {
      mimeType: 'text/csv',
      title: title,
    });
  } else {
    await saveText(text, title);
  }
}

async function saveText(text: string, title: string) {
  const filePath = await save({
    filters: [
      {
        name: 'CSV',
        extensions: ['csv'],
      },
    ],
    defaultPath: title + '.csv',
  });

  if (filePath) {
    await writeTextFile(filePath, text);
  }
}

import { shareText } from '@buildyourwebapp/tauri-plugin-sharesheet';
import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import { platform } from '@tauri-apps/plugin-os';

export enum ExportType {
  CSV,
  LOG,
}

export async function exportText(
  text: string,
  title: string,
  type: ExportType = ExportType.CSV,
) {
  const isMobile = platform() === 'ios' || platform() === 'android';

  if (isMobile) {
    await shareText(text, {
      mimeType: type === ExportType.CSV ? 'text/csv' : 'text/plain',
      title,
    });
    return true;
  }

  return await saveText(text, title, type);
}

async function saveText(
  text: string,
  title: string,
  type: ExportType = ExportType.CSV,
) {
  const filePath = await save({
    filters: [
      {
        name: type === ExportType.CSV ? 'CSV' : 'LOG',
        extensions: type === ExportType.CSV ? ['csv'] : ['log'],
      },
    ],
    defaultPath: title + (type === ExportType.CSV ? '.csv' : '.log'),
  });

  if (filePath) {
    await writeTextFile(filePath, text);
    return true;
  }

  return false;
}

import { shareText } from '@buildyourwebapp/tauri-plugin-sharesheet';
import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import { platform } from '@tauri-apps/plugin-os';

export enum ExportType {
  CSV,
  LOG,
  JSON,
}

const EXPORT_FORMATS = {
  [ExportType.CSV]: { name: 'CSV', extension: 'csv', mimeType: 'text/csv' },
  [ExportType.LOG]: { name: 'LOG', extension: 'log', mimeType: 'text/plain' },
  [ExportType.JSON]: {
    name: 'JSON',
    extension: 'json',
    mimeType: 'application/json',
  },
};

export async function exportText(
  text: string,
  title: string,
  type: ExportType = ExportType.CSV,
) {
  const isMobile = platform() === 'ios' || platform() === 'android';

  if (isMobile) {
    await shareText(text, {
      mimeType: EXPORT_FORMATS[type].mimeType,
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
  const format = EXPORT_FORMATS[type];
  const filePath = await save({
    filters: [{ name: format.name, extensions: [format.extension] }],
    defaultPath: `${title}.${format.extension}`,
  });

  if (filePath) {
    await writeTextFile(filePath, text);
    return true;
  }

  return false;
}

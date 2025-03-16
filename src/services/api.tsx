import { writeFile } from '@tauri-apps/plugin-fs';
import { join, tempDir} from '@tauri-apps/api/path';

interface AIResponse {
  response: string;
}

export const speechToText = async (audioBlob: Blob, scenarioId?: string): Promise<string> => {
  try {
    // 获取临时目录路径
    const tempDirStr = await tempDir();
    // 生成唯一的文件名
    const fileName = `audio_${Date.now()}.wav`;
    // 构建完整的文件路径
    const filePath = await join(tempDirStr, fileName);

    // 将 Blob 转换为 Uint8Array
    const arrayBuffer = await audioBlob.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);

    // 写入文件
    await writeFile(filePath, uint8Array);

    console.log('音频文件已保存到:', filePath);

    const formData = new FormData();
    formData.append('audio', audioBlob);
    if (scenarioId) formData.append('scenarioId', scenarioId);

    const response = await fetch('/api/speech-to-text', {
      method: 'POST',
      body: formData
    });
    return response.text();
  } catch (error) {
    console.error('保存音频文件失败:', error);
    throw error;
  }
};

export const getAIResponse = async (text: string): Promise<AIResponse> => {
  const response = await fetch('/api/get-ai-response', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ text })
  });
  return response.json();
};

export const textToSpeech = async (text: string): Promise<string> => {
  const response = await fetch('/api/text-to-speech', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ text })
  });
  return response.text();
}; 
import { invoke } from '@tauri-apps/api/core';
import { platform } from '@tauri-apps/plugin-os';
import { useState, useRef } from 'react';
import { SpeakerIcon } from '../components/Icons';
import { useParams } from 'react-router-dom';
import { speechToText, getAIResponse, textToSpeech } from '../services/api';

interface Message {
  type: 'user' | 'ai';
  text: string;
  audioUrl?: string;
}

const isInTauri = async () => {
  let platform_name = "unknown";
  try {
    platform_name = await platform();
    console.log(platform_name);
  } catch (e) {
    console.log("not in tauri");
  }
  return platform_name !== "unknown";
}

const ChatWindow = () => {
  const { scenarioId } = useParams<{ scenarioId: string }>();
  const [messages, setMessages] = useState<Message[]>([]);
  const [isRecording, setIsRecording] = useState(false);
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const audioChunksRef = useRef<Blob[]>([]);

  const startRecording = async () => {
    try {
      if (await isInTauri()) {
        // Check if running in Tauri
        await invoke('start_recording', {});
      } else {
        const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        mediaRecorderRef.current = new MediaRecorder(stream);
        audioChunksRef.current = [];

        mediaRecorderRef.current.ondataavailable = (event) => {
          audioChunksRef.current.push(event.data);
        };

        mediaRecorderRef.current.onstop = async () => {
          const audioBlob = new Blob(audioChunksRef.current);
          await processAudioMessage(audioBlob);
        };

        mediaRecorderRef.current.start();
      }
      setIsRecording(true);
    } catch (error) {
      console.error('录音失败:', error);
    }
  };

  const stopRecording = async () => {

    if (mediaRecorderRef.current && isRecording) {
      if (await isInTauri()) {
        await invoke('stop_recording');
      } else {
        mediaRecorderRef.current.stop();
      }
      setIsRecording(false);
    }
  };

  const processAudioMessage = async (audioBlob: Blob) => {
    try {
      const userText = await speechToText(audioBlob, scenarioId);

      // 添加用户消息
      setMessages(prev => [...prev, {
        type: 'user',
        text: userText,
      }]);

      const aiText = await getAIResponse(userText);

      const audioUrl = await textToSpeech(aiText.response);

      // 添加AI回复消息
      setMessages(prev => [...prev, {
        type: 'ai',
        text: aiText.response,
        audioUrl
      }]);
    } catch (error) {
      console.error('处理消息失败:', error);
    }
  };

  const playAudio = (audioUrl: string) => {
    const audio = new Audio(audioUrl);
    audio.play();
  };

  return (
    <div className="chat-window">
      <div className="messages-container">
        {messages.map((message, index) => (
          <div key={index} className={`message ${message.type}`}>
            <div className="message-bubble">
              {message.text}
            </div>
            {message.audioUrl && (
              <button
                className="audio-button"
                onClick={() => playAudio(message.audioUrl!)}
              >
                <SpeakerIcon />
              </button>
            )}
          </div>
        ))}
      </div>
      <button
        className={`record-button ${isRecording ? 'recording' : ''}`}
        onClick={isRecording ? stopRecording : startRecording}
      >
        {isRecording ? '停止录音' : '开始录音'}
      </button>
    </div>
  );
};

export default ChatWindow; 

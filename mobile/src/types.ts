export interface Photo {
  id: string;
  sessionId: string;
  sequence: number;
  webUrl: string;
  thumbnailUrl: string;
}

export interface Session {
  id: string;
  status: string;
  photoCount: number;
  kioskConnected: boolean;
  phoneConnected: boolean;
}

export interface WSMessage {
  type: string;
  data?: Record<string, unknown>;
}

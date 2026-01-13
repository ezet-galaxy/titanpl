interface HelloRequest {
  name?: string;
}

interface HelloResponse {
  message: string;
  timestamp: number;
}

export const hello = (req: TitanRequest<HelloRequest>): HelloResponse => {
  return {
    message: `Hello from Titan, ${req.body?.name}!`,
    timestamp: Date.now(),
  };
};
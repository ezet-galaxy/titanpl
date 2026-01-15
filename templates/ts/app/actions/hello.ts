interface HelloResponse {
    message: string;
}

export const hello = (req: TitanRequest): HelloResponse => {
    return {
        message: `Hello from Titan ${req.body.name || "World"}`,
    };
}

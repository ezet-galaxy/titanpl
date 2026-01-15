/**
 * @param {TitanRequest} req
 * @returns {{ message: string }}
 */
export const hello = (req) => {
   return {
        message: `Hello from Titan ${req.body.name}`,
    };
}

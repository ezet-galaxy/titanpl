/**
 * Titan - Framework for building web applications
 */

/**
 * Route handler for defining route responses
 */
export interface RouteHandler {
    /**
     * Send a direct response for the route
     * @param value - The response value (can be any type, object will be JSON)
     */
    reply(value: any): void;

    /**
     * Bind the route to a server-side action
     * @param name - The name of the action to execute
     */
    action(name: string): void;
}

/**
 * Main Titan application builder
 */
export interface Titan {
    /**
     * Define a GET route
     * @param route - The route path (e.g., '/users', '/posts/:id')
     * @returns Route handler for defining the response
     */
    get(route: string): RouteHandler;

    /**
     * Define a POST route
     * @param route - The route path (e.g., '/users', '/posts/:id')
     * @returns Route handler for defining the response
     */
    post(route: string): RouteHandler;

    /**
     * Log a message with module context
     * @param module - The module name for context
     * @param msg - The message to log
     */
    log(module: string, msg: string): void;

    /**
     * Start the Titan Server
     * @param port - The port to listen on (default: 3000)
     * @param msg - Optional startup message to display
     */
    start(port?: number, msg?: string): Promise<void>;
}

/**
 * Default Titan instance
 */
declare const t: Titan;

export default t;
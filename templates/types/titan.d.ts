/**
 * TITAN TYPE DEFINITIONS
 * ----------------------
 * These types are globally available in your Titan project.
 */

/**
 * The Titan Request Object passed to actions.
 */
interface TitanRequest<TBody = any, TParams = Record<string, string>, TQuery = Record<string, string>> {
  body: TBody;
  method: "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
  path: string;
  headers: TitanHeaders;
  params: TParams;
  query: TQuery;
}

interface TitanHeaders {
  host?: string;
  "content-type"?: string;
  "user-agent"?: string;
  authorization?: string;
  [key: string]: string | undefined;
}

interface DbConnection {
  /**
   * Execute a SQL query.
   * @param sql The SQL query string.
   * @param params Optional parameters for the query ($1, $2, etc).
   */
  query<T = any>(sql: string, params?: any[]): T[];
}

interface FetchOptions {
  method?: "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
  headers?: Record<string, string>;
  body?: string | object;
}

interface FetchResponse {
  ok: boolean;
  status?: number;
  body?: string;
  error?: string;
}

interface JwtSignOptions {
  expiresIn?: string | number;
}

/**
 * Route builder returned by t.get(), t.post(), etc.
 */
interface TitanRouteBuilder {
  /**
   * Reply with a static value (text or JSON)
   */
  reply(value: string | object): void;

  /**
   * Execute an action by name
   */
  action(name: string): void;
}

/**
 * Titan Runtime Interface
 */
interface TitanRuntime {
  // ============ ROUTING (app.ts) ============

  /**
   * Define a GET route
   */
  get(route: string): TitanRouteBuilder;

  /**
   * Define a POST route
   */
  post(route: string): TitanRouteBuilder;

  /**
   * Define a PUT route
   */
  put(route: string): TitanRouteBuilder;

  /**
   * Define a DELETE route
   */
  delete(route: string): TitanRouteBuilder;

  /**
   * Define a PATCH route
   */
  patch(route: string): TitanRouteBuilder;

  /**
   * Start the Titan server
   * @param port Port to listen on (default: 3000)
   * @param message Optional startup message
   */
  start(port?: number, message?: string): Promise<void>;

  // ============ UTILITIES (actions) ============

  /**
   * Log messages to the server console with Titan formatting.
   */
  log(module: string, message: string): void;

  /**
   * Read a file contents as string.
   * @param path Relative path to the file from project root.
   */
  read(path: string): string;

  /**
   * Make HTTP requests from actions
   */
  fetch(url: string, options?: FetchOptions): FetchResponse;

  /**
   * JWT utilities
   */
  jwt: {
    sign(payload: object, secret: string, options?: JwtSignOptions): string;
    verify<T = any>(token: string, secret: string): T;
  };

  /**
   * Password hashing utilities
   */
  password: {
    hash(password: string): string;
    verify(password: string, hash: string): boolean;
  };

  /**
   * Database utilities
   */
  db: {
    connect(url: string): DbConnection;
  };
}

/**
 * Define a Titan Action with type inference.
 * @example
 * export const hello = defineAction((req) => {
 *   return { message: `Hello ${req.body.name}` };
 * });
 */
declare function defineAction<
  TBody = any,
  TParams = Record<string, string>,
  TQuery = Record<string, string>,
  TResponse = any
>(
  actionFn: (req: TitanRequest<TBody, TParams, TQuery>) => TResponse
): (req: TitanRequest<TBody, TParams, TQuery>) => TResponse;

/**
 * Titan Runtime - Global instance
 */
declare const t: TitanRuntime;
import fs from "node:fs";

/** Starts the app. */
export function start(port) {
  return fs.createServer(port);
}

class Server {
  listen(port) {
    return port;
  }
}

import { api } from "./api";

/** User record. */
export interface User {
  id: string;
}

/** Fetches one user. */
export async function fetchUser(id: string): Promise<User> {
  return api.get(`/users/${id}`);
}

export const MAX_USERS: number = 50;

class LocalCache {
  get(id: string): User | undefined {
    return undefined;
  }
}

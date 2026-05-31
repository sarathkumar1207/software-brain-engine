import { AuthService } from "../auth/auth.service";

export function authMiddleware(token: string) {
  const service = new AuthService();
  return service.login(token);
}


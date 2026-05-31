import { authMiddleware } from "../middleware/auth.middleware";

export class AuthController {
  authenticate(token: string) {
    return authMiddleware(token);
  }
}


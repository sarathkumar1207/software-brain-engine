import { verifyJwt } from "./jwt.strategy";

export class AuthService {
  login(token: string) {
    return verifyJwt(token);
  }
}


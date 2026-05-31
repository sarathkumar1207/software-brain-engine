export class JwtStrategy {
  validate(token: string) {
    return token.length > 0;
  }
}

export function verifyJwt(token: string) {
  const strategy = new JwtStrategy();
  return strategy.validate(token);
}


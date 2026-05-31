import { helper } from "./helper";

export interface User {
  name: string;
}

export class Service {
  run(value: number) {
    return helper(value);
  }
}

export function app() {
  const service = new Service();
  return service.run(41);
}


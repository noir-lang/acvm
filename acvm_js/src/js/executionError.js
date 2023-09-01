export class ExecutionError extends Error {
  constructor(message, callStack) {
    super(message);
    this.callStack = callStack;
  }
}

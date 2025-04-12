import { AppError } from './app_error.mjs';
let triggersRegister = {}

export const TriggerRegister = {
  register(id, fn) {
    if (triggersRegister[id]) {
      throw AppError.internalError(`Trigger ${id} is already registered`);
    }

    triggersRegister[id] = fn;
  },

  ids() {
    return Object.keys(triggersRegister);
  },

  async call (context) {
    if (!(context.triggerId in triggersRegister)) {
      const msg = `Trigger ${context.triggerId} is not registered`;
      throw AppError.internalError(msg);
    }

    const fn = triggersRegister[context.triggerId];
    return await fn(context);
  }
}

import { AppError } from './app_error.mjs';
let actionsRegister = {}

export const ActionRegister = {
  register(id, fn) {
    if (actionsRegister[id]) {
      throw AppError.internalError(`Action ${id} is already registered`);
    }

    actionsRegister[id] = fn;
  },

  ids() {
    return Object.keys(actionsRegister);
  },

  async call (context) {
    if (!(context.actionId in actionsRegister)) {
      const msg = `Action ${context.actionId} is not registered`;
      throw AppError.internalError(msg);
    }

    const fn = actionsRegister[context.actionId];
    return await fn(context);
  }
}

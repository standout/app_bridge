import { AppError } from './app_error.mjs';

let actionsRegister = {}

export const ActionRegister = {
  register(id, fn, inputSchema, outputSchema) {
    if (actionsRegister[id]) {
      throw AppError.internalError(`Action ${id} is already registered`);
    }

    actionsRegister[id] = { fn, inputSchema, outputSchema };
  },

  ids() {
    return Object.keys(actionsRegister);
  },

  inputSchema(actionId) {
    if (!(actionId in actionsRegister)) {
      const msg = `Action ${actionId} is not registered`;
      throw AppError.internalError(msg);
    }
    return actionsRegister[actionId].inputSchema;
  },

  outputSchema(actionId) {
    if (!(actionId in actionsRegister)) {
      const msg = `Action ${actionId} is not registered`;
      throw AppError.internalError(msg);
    }
    return actionsRegister[actionId].outputSchema;
  },

  async call (context) {
    if (!(context.actionId in actionsRegister)) {
      const msg = `Action ${context.actionId} is not registered`;
      throw AppError.internalError(msg);
    }

    const def = actionsRegister[context.actionId];
    return await def.fn(context);
  }
}

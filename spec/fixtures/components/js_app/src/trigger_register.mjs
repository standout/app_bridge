import { AppError } from './app_error.mjs';
let triggersRegister = {}

export const TriggerRegister = {
  register(id, fn, inputSchema, outputSchema) {
    if (triggersRegister[id]) {
      throw AppError.internalError(`Trigger ${id} is already registered`);
    }

    triggersRegister[id] = { fn, inputSchema, outputSchema };
  },

  ids() {
    return Object.keys(triggersRegister);
  },

  inputSchema(triggerId) {
    if (!(triggerId in triggersRegister)) {
      const msg = `Trigger ${triggerId} is not registered`;
      throw AppError.internalError(msg);
    }
    return triggersRegister[triggerId].inputSchema;
  },

  outputSchema(triggerId) {
    if (!(triggerId in triggersRegister)) {
      const msg = `Trigger ${triggerId} is not registered`;
      throw AppError.internalError(msg);
    }
    return triggersRegister[triggerId].outputSchema;
  },

  async call (context) {
    if (!(context.triggerId in triggersRegister)) {
      const msg = `Trigger ${context.triggerId} is not registered`;
      throw AppError.internalError(msg);
    }

    const def = triggersRegister[context.triggerId];
    return await def.fn(context);
  }
}

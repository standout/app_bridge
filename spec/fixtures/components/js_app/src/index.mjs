import { TriggerRegister } from './trigger_register.mjs';
import { triggerBuilder } from './trigger_builder.mjs';

TriggerRegister.register("new-todos", triggerBuilder("todos"));
TriggerRegister.register("new-posts", triggerBuilder("posts"));
TriggerRegister.register("new-comments", triggerBuilder("comments"));
TriggerRegister.register("new-albums", triggerBuilder("albums"));
TriggerRegister.register("new-photos", triggerBuilder("photos"));
TriggerRegister.register("new-users", triggerBuilder("users"));

export const triggers = {
  triggerIds() {
    return TriggerRegister.ids();
  },

  async fetchEvents(context) {
    return await TriggerRegister.call(context)
  }
};

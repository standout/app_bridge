import { TriggerRegister } from './trigger_register.mjs';
import { triggerBuilder } from './trigger_builder.mjs';
import { ActionRegister } from './action_register.mjs';
import { actionBuilder } from './action_builder.mjs';

// Example trigger schemas
const triggerInputSchema = `{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "since": { "type": "string", "description": "Fetch events since ISO timestamp" }
  },
  "additionalProperties": false
}`;

const triggerOutputSchema = `{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "events": { "type": "array" },
    "store": { "type": "string" }
  }
}`;

TriggerRegister.register("new-todos", triggerBuilder("todos"), triggerInputSchema, triggerOutputSchema);
TriggerRegister.register("new-posts", triggerBuilder("posts"), triggerInputSchema, triggerOutputSchema);
TriggerRegister.register("new-comments", triggerBuilder("comments"), triggerInputSchema, triggerOutputSchema);
TriggerRegister.register("new-albums", triggerBuilder("albums"), triggerInputSchema, triggerOutputSchema);
TriggerRegister.register("new-photos", triggerBuilder("photos"), triggerInputSchema, triggerOutputSchema);
TriggerRegister.register("new-users", triggerBuilder("users"), triggerInputSchema, triggerOutputSchema);

// HTTP GET action schema
const httpGetInputSchema = `{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "url": {
      "type": "string",
      "format": "uri",
      "description": "The URL to make a GET request to"
    }
  },
  "required": ["url"],
  "additionalProperties": false
}`;

const httpGetOutputSchema = `{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "url": {
      "type": "string",
      "description": "The URL that was requested"
    },
    "response": {
      "type": "object",
      "description": "The parsed JSON response from the HTTP request"
    }
  },
  "required": ["url", "response"]
}`;

// HTTP POST action schema
const httpPostInputSchema = `{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "url": {
      "type": "string",
      "format": "uri",
      "description": "The URL to make a POST request to"
    },
    "body": {
      "type": "string",
      "format": "code",
      "description": "The JSON body to send with the POST request"
    }
  },
  "required": ["url"],
  "additionalProperties": false
}`;

const httpPostOutputSchema = `{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "url": {
      "type": "string",
      "description": "The URL that was requested"
    },
    "body": {
      "type": "object",
      "description": "The body that was sent with the request"
    },
    "response": {
      "type": "object",
      "description": "The parsed JSON response from the HTTP request"
    }
  },
  "required": ["url", "body", "response"]
}`;

// Complex input action schema
const complexInputSchema = `{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "customer": {
      "type": "object",
      "properties": {
        "status": {
          "type": "string",
          "enum": ["active", "inactive", "pending"]
        },
        "orders": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "items": {
                "type": "array",
                "items": {
                  "type": "object",
                  "properties": {
                    "sku": { "type": "string" },
                    "quantity": { "type": "integer" }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}`;

const complexInputOutputSchema = `{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "customer": {
      "type": "object",
      "description": "Processed customer information"
    },
    "processed": {
      "type": "boolean",
      "description": "Whether the input was processed successfully"
    }
  },
  "required": ["customer", "processed"]
}`;

ActionRegister.register("http-get", actionBuilder("get"), httpGetInputSchema, httpGetOutputSchema);
ActionRegister.register("http-post", actionBuilder("post"), httpPostInputSchema, httpPostOutputSchema);
ActionRegister.register("complex-input", actionBuilder("complex"), complexInputSchema, complexInputOutputSchema);

export const triggers = {
  triggerIds() {
    return TriggerRegister.ids();
  },

  inputSchema(triggerId) {
    return TriggerRegister.inputSchema(triggerId);
  },

  outputSchema(triggerId) {
    return TriggerRegister.outputSchema(triggerId);
  },

  async fetchEvents(context) {
    return await TriggerRegister.call(context)
  }
};

export const actions = {
  actionIds() {
    return ActionRegister.ids();
  },

  inputSchema(actionId) {
    return ActionRegister.inputSchema(actionId);
  },

  outputSchema(actionId) {
    return ActionRegister.outputSchema(actionId);
  },

  async execute(context) {
    return await ActionRegister.call(context)
  }
};

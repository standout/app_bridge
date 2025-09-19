import { TriggerRegister } from './trigger_register.mjs';
import { triggerBuilder } from './trigger_builder.mjs';
import { ActionRegister } from './action_register.mjs';
import { actionBuilder } from './action_builder.mjs';

// Example trigger schemas
const triggerInputSchema = `{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "include_extra_data": {
      "type": "boolean",
      "description": "Whether to include additional data in the response"
    }
  },
  "required": ["include_extra_data"],
  "additionalProperties": false
}`;

const triggerOutputSchema = `{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "events": { "type": "array" },
    "store": { "type": "string" }
  },
  "required": ["events", "store"],
  "additionalProperties": false
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
      "description": "The parsed JSON response from the HTTP request",
      "properties": {
        "status": {
          "type": "integer",
          "description": "HTTP status code"
        },
        "headers": {
          "type": "object",
          "description": "Response headers",
          "additionalProperties": {
            "type": "string"
          }
        },
        "data": {
          "type": "object",
          "description": "Response data",
          "additionalProperties": true
        }
      },
      "required": ["status"],
      "additionalProperties": true
    }
  },
  "required": ["url", "response"],
  "additionalProperties": false
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
      "description": "The body that was sent with the request",
      "properties": {
        "content": {
          "type": "string",
          "description": "Request body content"
        },
        "content_type": {
          "type": "string",
          "description": "Content type of the request"
        }
      },
      "additionalProperties": true
    },
    "response": {
      "type": "object",
      "description": "The parsed JSON response from the HTTP request",
      "properties": {
        "status": {
          "type": "integer",
          "description": "HTTP status code"
        },
        "headers": {
          "type": "object",
          "description": "Response headers",
          "additionalProperties": {
            "type": "string"
          }
        },
        "data": {
          "type": "object",
          "description": "Response data",
          "additionalProperties": true
        }
      },
      "required": ["status"],
      "additionalProperties": true
    }
  },
  "required": ["url", "body", "response"],
  "additionalProperties": false
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
    },
    "metadata": {
      "type": "object",
      "title": "Custom Metadata",
      "description": "Additional metadata as key-value pairs",
      "propertyNames": {
        "type": "string",
        "title": "Field Name"
      },
      "additionalProperties": {
        "type": "string",
        "title": "Field Value"
      }
    }
  },
  "required": ["customer"],
  "additionalProperties": false
}`;

const complexInputOutputSchema = `{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "customer": {
      "type": "object",
      "description": "Customer information with status and order history",
      "properties": {
        "status": {
          "type": "string",
          "enum": ["active", "inactive", "pending"],
          "description": "Current status of the customer account"
        },
        "orders": {
          "type": "array",
          "description": "List of customer orders",
          "items": {
            "type": "object",
            "description": "Individual order containing items",
            "properties": {
              "items": {
                "type": "array",
                "description": "Items within this order",
                "items": {
                  "type": "object",
                  "description": "Individual item in the order",
                  "properties": {
                    "sku": {
                      "type": "string",
                      "description": "Stock Keeping Unit identifier for the product"
                    },
                    "quantity": {
                      "type": "integer",
                      "description": "Number of units of this item ordered"
                    }
                  }
                }
              }
            }
          }
        }
      }
    },
    "metadata": {
      "type": "object",
      "title": "Custom Metadata",
      "description": "Additional metadata as key-value pairs",
      "propertyNames": {
        "type": "string",
        "title": "Field Name"
      },
      "additionalProperties": {
        "type": "string",
        "title": "Field Value"
      }
    },
    "environment_variables": {
      "type": "object",
      "description": "Environment variables passed to the app at runtime",
      "propertyNames": {
        "type": "string",
        "title": "Variable Name"
      },
      "additionalProperties": {
        "type": "string",
        "title": "Variable Value"
      }
    }
  },
  "required": ["customer", "environment_variables"],
  "additionalProperties": false
}`;

ActionRegister.register("http-get", actionBuilder("get"), httpGetInputSchema, httpGetOutputSchema);
ActionRegister.register("http-post", actionBuilder("post"), httpPostInputSchema, httpPostOutputSchema);
ActionRegister.register("complex-input", actionBuilder("complex"), complexInputSchema, complexInputOutputSchema);

export const triggers = {
  triggerIds() {
    return TriggerRegister.ids();
  },

  inputSchema(context) {
    try {
      const accountData = JSON.parse(context.account.serializedData);
      if (accountData.custom === true && context.triggerId === "new-posts") {
        // Return enhanced schema with custom field for new-posts trigger
        return JSON.stringify({
          "$schema": "https://json-schema.org/draft/2020-12/schema",
          "type": "object",
          "properties": {
            "include_extra_data": { "type": "boolean", "description": "Whether to include additional data in the response" },
            "include_custom_data": { "type": "boolean", "description": "Whether to include custom data for premium accounts" },
            "test_string": { "type": "string", "description": "A test string field for the new-posts trigger" }
          },
          "required": ["include_extra_data"],
          "additionalProperties": false
        });
      }
    } catch (e) {
      // Ignore parsing errors and fall back to base schema
    }

    // Return base schema
    return TriggerRegister.inputSchema(context.triggerId);
  },

  outputSchema(context) {
    try {
      const accountData = JSON.parse(context.account.serializedData);
      if (accountData.custom === true && context.triggerId === "new-posts") {
          // Return enhanced schema with custom metadata for new-posts trigger
          return JSON.stringify({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "properties": {
              "events": { "type": "array" },
              "store": { "type": "string" },
              "custom_metadata": {
                "type": "object",
                "description": "Additional metadata for premium accounts",
                "properties": {
                  "priority": {
                    "type": "string",
                    "enum": ["low", "medium", "high"],
                    "description": "Priority level for the trigger"
                  },
                  "tags": {
                    "type": "array",
                    "items": {
                      "type": "string"
                    },
                    "description": "Tags associated with the trigger"
                  }
                },
                "additionalProperties": false
              }
            },
            "required": ["events", "store"],
            "additionalProperties": false
          });
        }
    } catch (e) {
      // Ignore parsing errors and fall back to base schema
    }

    // Return base schema
    return TriggerRegister.outputSchema(context.triggerId);
  },

  async fetchEvents(context) {
    return await TriggerRegister.call(context)
  }
};

export const actions = {
  actionIds() {
    return ActionRegister.ids();
  },

  inputSchema(context) {
    try {
      const accountData = JSON.parse(context.account.serializedData);
      if (accountData.custom === true && context.actionId === "http-post") {
          // Return enhanced schema with custom headers for http-post action
          return JSON.stringify({
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
              },
              "custom_headers": {
                "type": "object",
                "description": "Custom headers for premium accounts",
                "properties": {
                  "authorization": {
                    "type": "string",
                    "description": "Authorization header value"
                  },
                  "x_custom_id": {
                    "type": "string",
                    "description": "Custom identifier header"
                  }
                },
                "additionalProperties": false
              }
            },
            "required": ["url"],
            "additionalProperties": false
          });
        }

        if (accountData.custom === true && context.actionId === "complex-input") {
          // Return enhanced schema with custom field for complex-input action
          return JSON.stringify({
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
              },
              "metadata": {
                "type": "object",
                "title": "Custom Metadata",
                "description": "Additional metadata as key-value pairs",
                "propertyNames": {
                  "type": "string",
                  "title": "Field Name"
                },
                "additionalProperties": {
                  "type": "string",
                  "title": "Field Value"
                }
              },
              "custom_string": {
                "type": "string",
                "description": "A custom string field for complex-input action"
              }
            },
            "required": ["customer"],
            "additionalProperties": false
          });
        }
    } catch (e) {
      // Ignore parsing errors and fall back to base schema
    }

    // Return base schema
    return ActionRegister.inputSchema(context.actionId);
  },

  outputSchema(context) {
    try {
      const accountData = JSON.parse(context.account.serializedData);
      if (accountData.custom === true && context.actionId === "http-post") {
          // Return enhanced schema with custom metadata for http-post action
          return JSON.stringify({
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
              },
              "custom_metadata": {
                "type": "object",
                "description": "Additional metadata for premium accounts",
                "properties": {
                  "execution_time": {
                    "type": "number",
                    "description": "Time taken to execute the action in milliseconds"
                  },
                  "rate_limit_info": {
                    "type": "object",
                    "properties": {
                      "remaining": {
                        "type": "integer",
                        "description": "Remaining API calls"
                      },
                      "reset_time": {
                        "type": "string",
                        "format": "date-time",
                        "description": "When the rate limit resets"
                      }
                    },
                    "additionalProperties": false
                  }
                },
                "additionalProperties": false
              }
            },
            "required": ["url", "body", "response"],
            "additionalProperties": false
          });
        }
      } catch (e) {
      // Ignore parsing errors and fall back to base schema
    }

    // Return base schema
    return ActionRegister.outputSchema(context.actionId);
  },

  async execute(context) {
    return await ActionRegister.call(context)
  }
};

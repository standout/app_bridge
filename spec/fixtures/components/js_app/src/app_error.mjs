
export const AppError = {
  _build(tag, message) {
    return {
        code: { tag },
        message,
      };
  },

  unauthenticated(message = "Authentication failed") {
    return this._build("unauthenticated", message);
  },

  forbidden(message = "Access denied") {
    return this._build("Authorization failed", message);
  },

  misconfigured(message = "Misconfigured integration") {
    return this._build("misconfigured", message);
  },

  unsupported(message = "The target system does not support this") {
    return this._build("unsupported", message);
  },

  rateLimit(message = "Rate limit exceeded at the target system") {
    return this._build("rate-limit", message);
  },

  timeout(message = "Timeout at the target system") {
    return this._build("timeout", message);
  },

  unavailable(message = "The target system is unavailable") {
    return this._build("unavailable", message);
  },

  internalError(message = "Internal error in the app") {
    return this._build("internal-error", message);
  },

  malformedResponse(message = "Malformed response from the target system") {
    return this._build("malformed-response", message);
  },

  other(message = "Unknown error") {
    return this._build("other", message);
  },
};

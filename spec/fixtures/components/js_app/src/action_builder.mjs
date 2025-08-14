import { RequestBuilder } from 'standout:app/http@2.0.1';
import { AppError } from './app_error.mjs';

export const actionBuilder = (resource) => {
  return async (context) => {
    try {
      const input = JSON.parse(context.serializedInput);
      const url = input.url;

      if (!url) {
        throw AppError.misconfigured("Missing 'url' in input");
      }

      let builder = new RequestBuilder()
        .url(url);

      // Configure method and body based on resource type
      if (resource === "post") {
        const body = input.body || "";
        builder = builder.method({ tag: "post" }).body(body);
      } else {
        builder = builder.method({ tag: "get" });
      }

      let response;
      try {
        response = await builder.send();
      } catch (e) {
        throw AppError.other(`Request failed: ${e.message}`);
      }

      return {
        serializedOutput: response.body
      };
    } catch (error) {
      throw AppError.other("Error performing action");
    }
  }
}

import { RequestBuilder } from 'standout:app/http@2.0.1';
import { AppError } from './app_error.mjs';

export const actionBuilder = (resource) => {
  return async (context) => {
    try {
      const input = JSON.parse(context.serializedInput);

      if (resource === "complex") {
        const customer = input.customer;
        if (!customer) {
          throw AppError.misconfigured("Missing 'customer' in input");
        }

        const output = {
          customer: customer,
          processed: true
        };

        return {
          serializedOutput: JSON.stringify(output)
        };
      }

      const url = input.url;
      if (!url) {
        throw AppError.misconfigured("Missing 'url' in input");
      }

      let builder = new RequestBuilder()
        .url(url);

      let bodyValue;
      // Configure method and body based on resource type
      if (resource === "post") {
        bodyValue = input.body || "";
        builder = builder.method({ tag: "post" }).body(bodyValue);
      } else {
        builder = builder.method({ tag: "get" });
      }

      let response;
      try {
        response = await builder.send();
      } catch (e) {
        throw AppError.other(`Request failed: ${e.message}`);
      }

      // Build output based on resource type
      let output;
      let responseData;
      try {
        responseData = JSON.parse(response.body);
      } catch (e) {
        throw AppError.other(`Invalid JSON response: ${e.message}`);
      }

      if (resource === "post") {
        output = {
          url: url,
          body: bodyValue,
          response: responseData
        };
      } else {
        output = {
          url: url,
          response: responseData
        };
      }

      return {
        serializedOutput: JSON.stringify(output)
      };
    } catch (error) {
      throw AppError.other("Error performing action");
    }
  }
}

import { RequestBuilder } from 'standout:app/http@4.0.0';
import { envVars, envVar } from 'standout:app/environment@4.0.0';
import { normalize } from 'standout:app/file@4.0.0';
import { AppError } from './app_error.mjs';

export const actionBuilder = (resource) => {
  return async (context) => {
    try {
      const input = JSON.parse(context.serializedInput);

      // File normalize action - normalizes any source (URL, base64, data URI) to FileData
      if (resource === "file-normalize") {
        // Get the source (can be URL, base64, or data URI)
        const source = input.source || input.url || input.base64 || input.data_uri;
        if (!source) {
          throw AppError.misconfigured("Missing 'source' in input");
        }

        // Get optional headers (array of [key, value] tuples) for URL requests
        const headers = input.headers || null;
        // Get optional filename override
        const filename = input.filename || null;

        // Use file.normalize - automatically detects input type
        // Returns { base64, contentType, filename }
        // The platform will find fields with format: "file-output" in the schema
        // and replace this data with the blob ID from file_uploader
        const fileData = normalize(source, headers, filename);

        return {
          serializedOutput: JSON.stringify({
            file: {
              base64: fileData.base64,
              content_type: fileData.contentType,
              filename: fileData.filename
            }
          })
        };
      }

      if (resource === "complex") {
        const customer = input.customer;
        if (!customer) {
          throw AppError.misconfigured("Missing 'customer' in input");
        }

        const environmentVariables = {};

        try {
          // Use the WASI environment interface to get environment variables
          const envVarsList = envVars();
          if (envVarsList && Array.isArray(envVarsList) && envVarsList.length > 0) {
            for (const [key, value] of envVarsList) {
              environmentVariables[key] = value;
            }
          } else {
            environmentVariables["message"] = "No environment variables returned from WASI interface";
          }
        } catch (error) {
          console.log("Error accessing environment variables:", error);
          environmentVariables["message"] = "Error accessing environment variables via WASI";
        }

        const output = {
          customer: customer,
          processed: true,
          environment_variables: environmentVariables
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

      let responseData;

      // Check if we're in test mode (mock HTTP requests)
      let isTestMode = false;
      try {
        const testModeValue = envVar('APP_BRIDGE_TEST_MODE');
        isTestMode = testModeValue !== null;
      } catch (error) {
        // If envVar fails, assume not in test mode
        isTestMode = false;
      }

      if (isTestMode) {
        // Return mock response data for tests
        if (resource === "post") {
          responseData = {
            "args": {},
            "data": bodyValue || "",
            "files": {},
            "form": {},
            "headers": {
              "Accept": "*/*",
              "Content-Length": (bodyValue || "").length.toString(),
              "Content-Type": "application/json",
              "Host": "mock.test",
              "User-Agent": "MockHTTP/1.0"
            },
            "json": bodyValue ? JSON.parse(bodyValue) : null,
            "origin": "127.0.0.1",
            "url": url
          };
        } else {
          responseData = {
            "args": {},
            "headers": {
              "Accept": "*/*",
              "Host": "mock.test",
              "User-Agent": "MockHTTP/1.0"
            },
            "origin": "127.0.0.1",
            "url": url
          };
        }
      } else {
        // Make the actual HTTP request
        let response;
        try {
          response = await builder.send();
        } catch (e) {
          throw AppError.other(`Request failed: ${e.message}`);
        }

        try {
          responseData = JSON.parse(response.body);
        } catch (e) {
          throw AppError.other(`Invalid JSON response: ${e.message}`);
        }
      }

      // Build output based on resource type
      let output;

      if (resource === "post") {
        output = {
          url: url,
          body: {
            content: bodyValue || "",
            content_type: "application/json"
          },
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

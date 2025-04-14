import { RequestBuilder } from 'standout:app/http@2.0.0';
import { AppError } from './app_error.mjs';
import { safeInt } from './utils.mjs';

export const triggerBuilder = (resource) => {
  return async (context) => {
    let offset = safeInt(context.store);
    let limit = 10;

    try {
      let builder = new RequestBuilder()
        .method({ tag: "get" })
        .url(`https://jsonplaceholder.typicode.com/${resource}`)

      let response;
      try {
        response = await builder.send();
      } catch (e) {
        throw AppError.other(`Error fetching events: ${e.message}`);
      }

      const objects = JSON.parse(response.body);
      const events = objects.slice(offset, limit).map((t) => ({
        id: String(t.id),
        serializedData: JSON.stringify(t),
      }));

      return {
        events: events,
        store:  String(offset + limit),
      };
    } catch (e) {
      throw AppError.other(`Error fetching events: ${e.message}`);
    }
  }
}

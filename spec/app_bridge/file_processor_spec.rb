# frozen_string_literal: true

RSpec.describe AppBridge::FileProcessor do
  let(:file_data) do
    {
      "base64" => "SGVsbG8gV29ybGQ=",
      "content_type" => "text/plain",
      "filename" => "hello.txt"
    }
  end

  let(:blob_id) { "signed_blob_id_123" }

  before do
    AppBridge.file_uploader = ->(_file_data) { blob_id }
  end

  after do
    # rubocop:disable Style/NilLambda
    AppBridge.file_uploader = ->(_) { nil }
    # rubocop:enable Style/NilLambda
  end

  describe ".call" do
    context "with a simple file-output field" do
      let(:data) { { "attachment" => file_data } }
      let(:schema) do
        {
          "properties" => {
            "attachment" => {
              "type" => "string",
              "format" => "file-output"
            }
          }
        }
      end

      it "replaces file data with blob ID" do
        result = described_class.call(data, schema)
        expect(result["attachment"]).to eq(blob_id)
      end

      it "calls the file_uploader with file data" do
        expect(AppBridge.file_uploader).to receive(:call).with(
          hash_including("base64" => "SGVsbG8gV29ybGQ=", "filename" => "hello.txt")
        ).and_return(blob_id)

        described_class.call(data, schema)
      end
    end

    context "with nested file-output field" do
      let(:data) do
        {
          "invoice" => {
            "attachment" => file_data,
            "number" => "INV-001"
          }
        }
      end

      let(:schema) do
        {
          "properties" => {
            "invoice" => {
              "properties" => {
                "attachment" => {
                  "type" => "string",
                  "format" => "file-output"
                },
                "number" => {
                  "type" => "string"
                }
              }
            }
          }
        }
      end

      it "replaces nested file data with blob ID" do
        result = described_class.call(data, schema)
        expect(result["invoice"]["attachment"]).to eq(blob_id)
        expect(result["invoice"]["number"]).to eq("INV-001")
      end
    end

    context "with array of file-output fields" do
      let(:second_file_data) do
        {
          "base64" => "V29ybGQgSGVsbG8=",
          "content_type" => "text/plain",
          "filename" => "world.txt"
        }
      end

      let(:data) do
        {
          "attachments" => [file_data, second_file_data]
        }
      end

      let(:schema) do
        {
          "properties" => {
            "attachments" => {
              "type" => "array",
              "items" => {
                "type" => "string",
                "format" => "file-output"
              }
            }
          }
        }
      end

      it "replaces all file data in array with blob IDs" do
        call_count = 0
        AppBridge.file_uploader = lambda { |_data|
          call_count += 1
          "blob_#{call_count}"
        }

        result = described_class.call(data, schema)
        expect(result["attachments"]).to eq(%w[blob_1 blob_2])
      end
    end

    context "with non-file field" do
      let(:data) do
        {
          "name" => "Test",
          "count" => 42
        }
      end

      let(:schema) do
        {
          "properties" => {
            "name" => { "type" => "string" },
            "count" => { "type" => "integer" }
          }
        }
      end

      it "leaves non-file fields unchanged" do
        result = described_class.call(data, schema)
        expect(result["name"]).to eq("Test")
        expect(result["count"]).to eq(42)
      end
    end

    context "with missing file data" do
      let(:data) { { "attachment" => nil } }
      let(:schema) do
        {
          "properties" => {
            "attachment" => {
              "type" => "string",
              "format" => "file-output"
            }
          }
        }
      end

      it "leaves nil values unchanged" do
        result = described_class.call(data, schema)
        expect(result["attachment"]).to be_nil
      end
    end

    context "with invalid file data (missing required fields)" do
      let(:data) { { "attachment" => { "base64" => "data" } } }
      let(:schema) do
        {
          "properties" => {
            "attachment" => {
              "type" => "string",
              "format" => "file-output"
            }
          }
        }
      end

      it "leaves invalid file data unchanged" do
        result = described_class.call(data, schema)
        expect(result["attachment"]).to eq({ "base64" => "data" })
      end
    end

    context "when uploader raises an error" do
      let(:data) { { "attachment" => file_data } }
      let(:schema) do
        {
          "properties" => {
            "attachment" => {
              "type" => "string",
              "format" => "file-output"
            }
          }
        }
      end

      before do
        AppBridge.file_uploader = ->(_) { raise StandardError, "Upload failed" }
      end

      it "raises AppBridge::InternalError with filename in message" do
        expect { described_class.call(data, schema) }
          .to raise_error(AppBridge::InternalError, "Failed to upload 'hello.txt': Upload failed")
      end
    end

    context "when uploader returns nil (no-op)" do
      let(:data) { { "attachment" => file_data } }
      let(:schema) do
        {
          "properties" => {
            "attachment" => {
              "type" => "string",
              "format" => "file-output"
            }
          }
        }
      end

      before do
        # rubocop:disable Style/NilLambda
        AppBridge.file_uploader = ->(_) { nil }
        # rubocop:enable Style/NilLambda
      end

      it "leaves file data unchanged" do
        result = described_class.call(data, schema)
        expect(result["attachment"]).to eq(file_data)
      end
    end

    context "with deeply nested structure" do
      let(:data) do
        {
          "order" => {
            "items" => [
              {
                "product" => "Widget",
                "documents" => {
                  "invoice" => file_data
                }
              }
            ]
          }
        }
      end

      let(:schema) do
        {
          "properties" => {
            "order" => {
              "properties" => {
                "items" => {
                  "type" => "array",
                  "items" => {
                    "properties" => {
                      "product" => { "type" => "string" },
                      "documents" => {
                        "properties" => {
                          "invoice" => {
                            "type" => "string",
                            "format" => "file-output"
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      end

      it "processes files in deeply nested structures" do
        result = described_class.call(data, schema)
        expect(result["order"]["items"][0]["documents"]["invoice"]).to eq(blob_id)
        expect(result["order"]["items"][0]["product"]).to eq("Widget")
      end
    end

    context "with real-world nested files structure (files.files array)" do
      let(:data) do
        {
          "aggregated-invoices" => {},
          "agreement-nr" => "",
          "amount" => 500,
          "files" => {
            "files" => [
              {
                "base64" => "SGVsbG8gV29ybGQ=",
                "content_type" => "application/pdf",
                "filename" => "test.pdf"
              }
            ]
          },
          "foreign-id" => "appakettest",
          "id" => 4_690_020
        }
      end

      let(:schema) do
        {
          "additionalProperties" => true,
          "properties" => {
            "aggregated-invoices" => {
              "properties" => {},
              "title" => "Samlingsfakturor",
              "type" => "object"
            },
            "agreement-nr" => { "title" => "Avtalsnummer", "type" => "string" },
            "amount" => { "title" => "Belopp", "type" => "integer" },
            "files" => {
              "properties" => {
                "files" => {
                  "items" => { "format" => "file-output", "title" => "Fil", "type" => "object" },
                  "title" => "Filer",
                  "type" => "array"
                }
              },
              "title" => "Filer",
              "type" => "object"
            },
            "foreign-id" => { "title" => "Externt ID", "type" => "string" },
            "id" => { "title" => "Faktura-ID", "type" => "integer" }
          },
          "type" => "object"
        }
      end

      it "processes files in nested files.files array structure" do
        result = described_class.call(data, schema)

        # The file should be replaced with blob_id
        expect(result["files"]["files"]).to eq([blob_id])

        # Other fields should be unchanged
        expect(result["amount"]).to eq(500)
        expect(result["foreign-id"]).to eq("appakettest")
        expect(result["id"]).to eq(4_690_020)
      end

      it "calls file_uploader with correct file data" do
        expect(AppBridge.file_uploader).to receive(:call).with(
          hash_including(
            "base64" => "SGVsbG8gV29ybGQ=",
            "content_type" => "application/pdf",
            "filename" => "test.pdf"
          )
        ).and_return(blob_id)

        described_class.call(data, schema)
      end
    end

    context "with symbol keys in data" do
      let(:data) do
        {
          attachment: {
            base64: "SGVsbG8gV29ybGQ=",
            content_type: "text/plain",
            filename: "hello.txt"
          }
        }
      end

      let(:schema) do
        {
          "properties" => {
            "attachment" => {
              "type" => "string",
              "format" => "file-output"
            }
          }
        }
      end

      it "handles symbol keys correctly by normalizing to string keys" do
        result = described_class.call(data, schema)
        # Keys are normalized to strings in the output
        expect(result["attachment"]).to eq(blob_id)
      end
    end
  end
end

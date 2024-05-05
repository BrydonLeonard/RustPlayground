# Manifest-o

Reads manifestos for political parties and sends them off to GPT-3.5/4 to be summarized. You'll need to get an API token from OpenAI and store that in a single line in a file _somewhere_; that'll be used to make the request to the OpenAI API.

To run from cargo:
```bash
cargo run -- test_input /path/to/a/file/with/you/openai/secret 
```



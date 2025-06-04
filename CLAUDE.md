The character.txt file contains information about llm behavior, we are going to build a system leveraging that behavior -- to get started we need a few things
1. low level client trait
2. given a low level client, implement retry logic on errors before forwarding the error to the caller, and verifies a particular json response output structure (the function has a generic return), if there is a failure implement retry logic which describes the failure mode (through the error) and send that as context to the model along with the previous message. Number of retries per type should be a configuration of the client
3. application level clients should now be able to make a call to the low level client, expecting a particular data type and ? unwrap it guaranteeing that data and retries or a QueryResolverError, which can contain AIError, which then contains the particular (claude|openai|etc)*(api error (http|idk|etc)) 
4. given the format of a ticket decomposition, we can almost get to work extracting specific information, 




but first we should take care of wrapping tickets in additional metadata, specifically metadata indicating this tickets relationship to other tickets.


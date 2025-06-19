We are working on the framework framework (f(fw)w)

to start we will be testing to see what kinds of tools we should build to abstract on top of the strongly typed llm interface in ../client-implementations

my reasoning tells me that we will begin toying with human-ai knowledge extrapolation (called tlamatini) and in doing so we will be looking into epistomoligical and statistical foundations like 

1.  how can we have confidence that generated facts (true or false) represent the underlying structure of the representation; ie. "the file doesn't exist" is true and reflects the non-existence of a file, "the code is tested and passes" is true and represents that the tests reflect the code, and the code reflects the spec, and the tests have been executed and return a passing score; "the code doesn't have vulnerabilities" is false and reflects that the code has an issue and more work must be done to eliminate it

2. how can we have confidence in the semantic meaning of generated facts? "generate qa pairs for this text" requires us to generate pairs of strings, and then verify that those strings represent qa pairs which reflect the text; this can be observed through recursive decomposition, where each pair is related to the text and verified -- but how can a system have confidence that any single pair isn't being hallucinated? we can utilize delegations of humans to manually view the generated data and provide their feedback on the semantic meaning and it's relation to the text; it may be possible to utilize delegations of cognitive systems (of a certain level of quality) to perform those verification steps

3. how can we construct knowledge verification chains which allow us to have confidence in high level facts like "the qa webapp works as expected?" this requires us to have definitions for expected behavior and systems for observing and verifying expected behavior ie. down one chain: webapp works -> backend works -> qa endpoint can generate new pairs given a document id -> a pair can be reliably generated on text given a particular cognitive agent and prompted through query_with_schema -> the claude cognitive agent can reliably return structured data -> integration tests pass. at any one of these steps, it can be observed that the verification chain has multiple branches. The construction of the verification lattice will be an important aspect of any reliable system

---

One possible direction for exploration after the todoapp-framework is a requirements engineering exploration. That being, can we go from a rough idea to a modular decomposition 

---

A higher order program:
For all of programming history humans have been responsible for building and maintaining a very narrow slice of possible programs. We never really wrote software which required human laborers, or at least any such software was not represented in traditional computers, but in business processes. Any cognitive input into a computer program was so slow and so unreliable, that cognition hasn't really ever made it into high throughput computer algorithms. 100 async threads would require 100 human executors. Now scale to the size of a social media empire. The human cost is simply too high to write cognition as a primitive.

Now however we are quickly aproaching a new world. One where software becomes a primitive. One where software is manipulated by language models, and generated on the fly to solve regular patterns in the effort of satisfying a high level goal.

Take for instance a bot designed to construct api's through a network scraping algorithms. A database filled with detailed scraping programs, satisfying a mapping "www.website.com" -> ("www.website.com.scraper.rs", website map). A website map is explored by the agent, performing a fuzzy search for a field of interest, if it's not there, the llm starts a scraper generator bot to sleuth the site, occassionally checking for honeypots through semantic observation. After some exploration the system observes that the field of interest has been discovered. All other required fields are found -- and an api is constructed to provide the values scraped from the site. The output to the user is simple
```
> gen_scraper www.upwork.com --purpose "construct an api for upwork.com to stream jobs"
Generating scraper for upwork.com
Found scraper for jobs
forwarding job scraper
http://scraper.ai/ah3lc9fh40348nf49v393h4bgn4938284h/com/upwork
downloading swagger.io...
```

we can take this further
```
universal_interface http://scraper.ai/ah3lc9fh40348nf49v393h4bgn4938284h/com/upwork
> ls
get jobs(/id) post jobs(/id)
> act "post an offer for a job that looks doable"
... constructing resume
... submitting
... done
```

This world is one where agents manage algorithms and algorithms manage agents. This is a world where computers write programs to solve their problems. This is a world where software can become a core primitive in the next generation of computer programs. 

-- to bring us closer to reality let me go back to the subject before. Requirements Engineering. At this phase my primitives aren't multi agent distributed systems and program databases, it's going to be datasets and their transformations. How is it that we take an idea -> artifact with the fidelity that would make a human gasp? What does it take to turn an idea into reality? And more importantly, what are the intermediate steps? 

Refinement is the business process by which an idea is separated from the non-ideas. Where a cognitive architect begins to cut and chisel and idea from the noise. When the mind contains a clear representation, then words can stream out -- a protocol who's express purpose is to produce a -- at best -- perfect fidelity image of the underlying structure being described. However, this is a problem so difficult, that sometimes to understand the difference between two ideas, you need to graze mathematical unknowability itself. Humans are very good problem solvers, principally because we are able to create abstractions which keep our knowledge distinct from the unknown. By the use of language humans build complex conceptual renderings inside of the minds of their listeners. How can we become like a prospector, mining their conceptual frameworks for structure. A lidar detector for the mind
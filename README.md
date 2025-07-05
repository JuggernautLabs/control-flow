# High Level 

This project is a hodgpodge of other project. The overall exploration is about uncovering the low level primitives that will be required in building a coherent system of agents. The high level idea is: ai corporations should be as easy to manage as individual agents. 

The key challenge is that the outputs of individual agents are never very clear. If I ask an agent to do something, like create a single file python application that does x,y, and z how can we be sure that when the agent terminates the output has been properly constructed? Furthermore, how can we be sure that the desires of the human have been properly represented? 

Let's start unpacking these ideas, but first we have to get comfortable considering cogition functions like
```
let result: Unverified<PythonApplication, PythonTests> = generate_single_file_python_application("make a todo app")
```

if we want to look into how this is produced, we must need multiple layers 

```
let initial_plan: Plan = refine_plan(initial_task)
let refined: Plan = initial_plan.append_tasks(inital_plan.unknowns.extrapolate_with_human_input())
let tests: PythonTests = refined.construct_test_file()
let correspondence: bool = is_related_enough(tests, refined)
let app: PythonApplication = build_application([tests,refined])
Unverified::new(PythonApplication, PythonTests)
```

you can see that there is a process for constructing a reliable single file python application from a combination of human inputs. what do we do once we have an unverified application?

```
let verified_syntax:bool = check_syntax(result.app)
if !verified_syntax {
  // iterate
}
let verified_tests: bool = result.tests.execute()
if !verified_test {
  //iterate
}
results.verify() // returns VerifiedApp<PythonApp, PythonTests>
```
# More Structure

This is no longer just theory; the designs are being worked out in ffww (currently private). The ideas there show a process by which we convert a string like "a rest enabled todo application" becomes an, often buggy, implementation in rust. In reality something like a single file rust application is likely one-shottable for modern llms; what this work shows is that llm outputs, when structured properly, can be deterministically assembled. More-over, llm's can derive verifiable, structured plans which can be implemented in parallel. The String -> RustProject pipeline shows that we can harness "Semantic Inflation" to turn vague ideas into real implementations.

# Further Research

1. Where in this pipeline is human interaction desired and/or required
2. What does generic pipeline infrastructure look like
3. What kinds of context is required, and how should prompts be structured to reliably query<T>
4. how should inter-chunk dependencies be handled
5. how should multi-module projects tersely encode module interfaces AND how can we trim those interfaces to cover only module dependencies
6. how should project-sized modules communicate in a multi-module environment
7. how can local requirements modify higher level constraints

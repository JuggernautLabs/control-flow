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
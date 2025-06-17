
```
 cargo run --example todoapp-framework
```

```
Running `target/debug/examples/todoapp-framework`
🎯 Goal: Build a simple command-line todo app
📋 Requirements extracted:
  Features: ["add new task with description", "list all tasks with their status and ID", "mark task as complete by ID", "delete task by ID", "clear all completed tasks", "display task count (total/completed)", "show help/usage instructions", "exit application"]
  Interface: command_line
  Storage: file
  Confidence: 0.80
💻 Application generated (3773 chars code, 1659 chars tests)
  Implementation confidence: 0.90
🔍 Semantic verification:
  Requirements fulfilled: true
  Tests adequate: true
  Correspondence score: 0.95
  Gaps identified: ["No explicit test for help/usage instructions display", "No test verifying the exact format of task listing output", "No error handling test cases for invalid file operations"]

✅ App built successfully!
Usage: To run the application:
1. Save the code as todo_app.py
2. Run `python todo_app.py` from the command line
3. Use the following commands:
   - 'a <description>' to add a new task
   - 'l' to list all tasks
   - 'c <id>' to mark a task as complete
   - 'd <id>' to delete a task
   - 'x' to clear completed tasks
   - 's' to show task statistics
   - 'h' for help
   - 'q' to quit

To run the tests:
1. Install pytest: `pip install pytest`
2. Save the test code as test_todo_app.py in the same directory as todo_app.py
3. Run `pytest test_todo_app.py`

The application stores tasks in a JSON file (tasks.json by default) in the same directory as the script.

Files written: todo_app.py, test_todo_app.py

🧪 Running tests...
Test stdout:
============================= test session starts ==============================
platform darwin -- Python 3.13.5, pytest-8.4.0, pluggy-1.6.0 -- /opt/homebrew/Cellar/pytest/8.4.0/libexec/bin/python
cachedir: .pytest_cache
rootdir: /Users/shane.mendez/dev/sandbox/control-flow/control-flow/ffww
collecting ... collected 6 items

test_todo_app.py::test_add_task PASSED                                   [ 16%]
test_todo_app.py::test_mark_complete PASSED                              [ 33%]
test_todo_app.py::test_delete_task PASSED                                [ 50%]
test_todo_app.py::test_clear_completed PASSED                            [ 66%]
test_todo_app.py::test_get_stats PASSED                                  [ 83%]
test_todo_app.py::test_persistence PASSED                                [100%]

============================== 6 passed in 0.01s ===============================

✅ Tests passed!

📊 Summary:
  Requirements confidence: 0.80
  Implementation confidence: 0.90
  Correspondence score: 0.95
  Tests passed: true
  AI predicted success: true
  Actual success: true
  Prediction accuracy: true
  Identified gaps: ["No explicit test for help/usage instructions display", "No test verifying the exact format of task listing output", "No error handling test cases for invalid file operations"]

```
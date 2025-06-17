import pytest
import os
from todo_app import TodoApp

@pytest.fixture
def todo_app():
    app = TodoApp('test_tasks.json')
    yield app
    if os.path.exists('test_tasks.json'):
        os.remove('test_tasks.json')

def test_add_task(todo_app):
    todo_app.add_task('Test task')
    tasks = todo_app.list_tasks()
    assert len(tasks) == 1
    assert tasks[0]['description'] == 'Test task'
    assert not tasks[0]['completed']

def test_mark_complete(todo_app):
    todo_app.add_task('Test task')
    task_id = todo_app.list_tasks()[0]['id']
    assert todo_app.mark_complete(task_id)
    assert todo_app.list_tasks()[0]['completed']

def test_delete_task(todo_app):
    todo_app.add_task('Test task')
    task_id = todo_app.list_tasks()[0]['id']
    assert todo_app.delete_task(task_id)
    assert len(todo_app.list_tasks()) == 0

def test_clear_completed(todo_app):
    todo_app.add_task('Task 1')
    todo_app.add_task('Task 2')
    task_id = todo_app.list_tasks()[0]['id']
    todo_app.mark_complete(task_id)
    todo_app.clear_completed()
    tasks = todo_app.list_tasks()
    assert len(tasks) == 1
    assert tasks[0]['description'] == 'Task 2'

def test_get_stats(todo_app):
    todo_app.add_task('Task 1')
    todo_app.add_task('Task 2')
    task_id = todo_app.list_tasks()[0]['id']
    todo_app.mark_complete(task_id)
    stats = todo_app.get_stats()
    assert stats['total'] == 2
    assert stats['completed'] == 1

def test_persistence(todo_app):
    todo_app.add_task('Persistent task')
    new_app = TodoApp('test_tasks.json')
    tasks = new_app.list_tasks()
    assert len(tasks) == 1
    assert tasks[0]['description'] == 'Persistent task'
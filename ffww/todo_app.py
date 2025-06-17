import json
import os
from typing import Dict, List

class TodoApp:
    def __init__(self, filename='tasks.json'):
        self.filename = filename
        self.tasks = self.load_tasks()

    def load_tasks(self) -> List[Dict]:
        if os.path.exists(self.filename):
            try:
                with open(self.filename, 'r') as f:
                    return json.load(f)
            except json.JSONDecodeError:
                return []
        return []

    def save_tasks(self) -> None:
        with open(self.filename, 'w') as f:
            json.dump(self.tasks, f)

    def add_task(self, description: str) -> None:
        task = {
            'id': len(self.tasks) + 1,
            'description': description,
            'completed': False
        }
        self.tasks.append(task)
        self.save_tasks()

    def list_tasks(self) -> List[Dict]:
        return self.tasks

    def mark_complete(self, task_id: int) -> bool:
        for task in self.tasks:
            if task['id'] == task_id:
                task['completed'] = True
                self.save_tasks()
                return True
        return False

    def delete_task(self, task_id: int) -> bool:
        for task in self.tasks:
            if task['id'] == task_id:
                self.tasks.remove(task)
                self.save_tasks()
                return True
        return False

    def clear_completed(self) -> None:
        self.tasks = [task for task in self.tasks if not task['completed']]
        self.save_tasks()

    def get_stats(self) -> Dict:
        total = len(self.tasks)
        completed = len([task for task in self.tasks if task['completed']])
        return {'total': total, 'completed': completed}

def main():
    app = TodoApp()
    
    while True:
        command = input('\nEnter command (h for help): ').strip().lower()
        
        if command == 'h':
            print('''
Commands:
  a <description> - Add new task
  l              - List all tasks
  c <id>         - Mark task as complete
  d <id>         - Delete task
  x              - Clear completed tasks
  s              - Show task statistics
  h              - Show this help
  q              - Quit
''')
        
        elif command.startswith('a '):
            app.add_task(command[2:])
            print('Task added')
        
        elif command == 'l':
            tasks = app.list_tasks()
            if not tasks:
                print('No tasks')
            for task in tasks:
                status = '✓' if task['completed'] else ' '
                print(f"[{status}] {task['id']}: {task['description']}")
        
        elif command.startswith('c '):
            try:
                task_id = int(command[2:])
                if app.mark_complete(task_id):
                    print('Task marked as complete')
                else:
                    print('Task not found')
            except ValueError:
                print('Invalid task ID')
        
        elif command.startswith('d '):
            try:
                task_id = int(command[2:])
                if app.delete_task(task_id):
                    print('Task deleted')
                else:
                    print('Task not found')
            except ValueError:
                print('Invalid task ID')
        
        elif command == 'x':
            app.clear_completed()
            print('Completed tasks cleared')
        
        elif command == 's':
            stats = app.get_stats()
            print(f"Total tasks: {stats['total']}")
            print(f"Completed tasks: {stats['completed']}")
        
        elif command == 'q':
            break
        
        else:
            print('Invalid command. Type h for help.')

if __name__ == '__main__':
    main()
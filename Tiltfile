docker_build('todo', '.', ignore=['target', 'todo'],
    dockerfile='./Dockerfile.dev',
    live_update=[
        sync('.', '/app'),
])
docker_compose('./docker-compose.yml')

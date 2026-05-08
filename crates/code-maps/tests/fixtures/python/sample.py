import os
from pathlib import Path

class Project:
    pass

async def load_project(path: Path) -> Project:
    return Project()

def _helper(count: int) -> int:
    return count + 1

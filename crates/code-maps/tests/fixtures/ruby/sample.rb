require "json"

class Project
  def initialize(name)
    @name = name
  end
end

def load_project(path)
  Project.new(path)
end

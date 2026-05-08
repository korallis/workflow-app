package sample

import "fmt"

type Project struct {
	Name string
}

func LoadProject(path string) (*Project, error) {
	return &Project{Name: path}, nil
}

func helper(count int) int {
	return count + 1
}

func (p *Project) NameLen() int {
	return len(p.Name)
}

func printProject(p Project) {
	fmt.Println(p.Name)
}

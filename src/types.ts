export interface Agent {
  id: string
  name: string
  skillsPath: string
  rulesPath: string
}

export interface Project {
  id: string
  path: string
  agentIds: string[]
}

export interface Skill {
  id: string
  name: string
  folder: string
  description: string | null
  filePath: string
  content: string
}

export interface PrStatus {
  state: string
  merged: boolean
}

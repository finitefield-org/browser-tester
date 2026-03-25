package runtime

func (s *Session) MainCount() int {
	return s.elementCountForSelector("main")
}

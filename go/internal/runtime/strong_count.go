package runtime

func (s *Session) StrongCount() int {
	return s.elementCountForSelector("strong")
}

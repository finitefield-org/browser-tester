package runtime

func (s *Session) SmallCount() int {
	return s.elementCountForSelector("small")
}

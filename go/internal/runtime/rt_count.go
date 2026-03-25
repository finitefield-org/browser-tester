package runtime

func (s *Session) RtCount() int {
	return s.elementCountForSelector("rt")
}

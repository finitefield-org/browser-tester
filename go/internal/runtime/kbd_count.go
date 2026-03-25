package runtime

func (s *Session) KbdCount() int {
	return s.elementCountForSelector("kbd")
}

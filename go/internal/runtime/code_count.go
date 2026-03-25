package runtime

func (s *Session) CodeCount() int {
	return s.elementCountForSelector("code")
}

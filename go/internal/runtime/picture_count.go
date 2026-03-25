package runtime

func (s *Session) PictureCount() int {
	return s.elementCountForSelector("picture")
}

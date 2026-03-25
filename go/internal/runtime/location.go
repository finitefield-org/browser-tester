package runtime

import (
	"fmt"
	"net"
	"net/url"
	"path"
	"strconv"
	"strings"
)

func (s *Session) SetLocationProperty(property, value string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	normalizedProperty := strings.ToLower(strings.TrimSpace(property))
	if normalizedProperty == "" {
		return fmt.Errorf("location property must not be empty")
	}

	resolved, err := resolveLocationPropertyURL(s.URL(), normalizedProperty, value)
	if err != nil {
		return err
	}
	return s.recordNavigation(resolved)
}

func (s *Session) LocationHref() (string, error) {
	if s == nil {
		return "", fmt.Errorf("session is unavailable")
	}
	return s.URL(), nil
}

func (s *Session) LocationOrigin() (string, error) {
	parsed, err := s.currentLocationURL()
	if err != nil {
		return "", err
	}
	if parsed.Scheme == "" || parsed.Host == "" {
		return "null", nil
	}
	return parsed.Scheme + "://" + parsed.Host, nil
}

func (s *Session) LocationProtocol() (string, error) {
	parsed, err := s.currentLocationURL()
	if err != nil {
		return "", err
	}
	if parsed.Scheme == "" {
		return "", nil
	}
	return parsed.Scheme + ":", nil
}

func (s *Session) LocationHost() (string, error) {
	parsed, err := s.currentLocationURL()
	if err != nil {
		return "", err
	}
	return parsed.Host, nil
}

func (s *Session) LocationHostname() (string, error) {
	parsed, err := s.currentLocationURL()
	if err != nil {
		return "", err
	}
	return parsed.Hostname(), nil
}

func (s *Session) LocationPort() (string, error) {
	parsed, err := s.currentLocationURL()
	if err != nil {
		return "", err
	}
	return parsed.Port(), nil
}

func (s *Session) LocationPathname() (string, error) {
	parsed, err := s.currentLocationURL()
	if err != nil {
		return "", err
	}
	pathname := parsed.EscapedPath()
	if pathname == "" {
		pathname = "/"
	}
	return pathname, nil
}

func (s *Session) LocationSearch() (string, error) {
	parsed, err := s.currentLocationURL()
	if err != nil {
		return "", err
	}
	if parsed.RawQuery == "" {
		if parsed.ForceQuery {
			return "?", nil
		}
		return "", nil
	}
	return "?" + parsed.RawQuery, nil
}

func (s *Session) LocationHash() (string, error) {
	parsed, err := s.currentLocationURL()
	if err != nil {
		return "", err
	}
	hash := parsed.EscapedFragment()
	if hash == "" {
		return "", nil
	}
	return "#" + hash, nil
}

func (s *Session) currentLocationURL() (*url.URL, error) {
	if s == nil {
		return nil, fmt.Errorf("session is unavailable")
	}
	raw := strings.TrimSpace(s.URL())
	if raw == "" {
		return nil, fmt.Errorf("location is unavailable")
	}
	parsed, err := url.Parse(raw)
	if err != nil {
		return nil, err
	}
	return parsed, nil
}

func resolveLocationPropertyURL(baseURL, property, value string) (string, error) {
	switch property {
	case "href":
		trimmed := strings.TrimSpace(value)
		if trimmed == "" {
			return "", fmt.Errorf("location.href requires a non-empty URL")
		}
		return resolveHyperlinkURL(baseURL, trimmed), nil
	case "hash":
		return mutateLocationURL(baseURL, func(parsed *url.URL) error {
			parsed.Fragment = strings.TrimLeft(strings.TrimSpace(value), "#")
			return nil
		})
	case "pathname":
		return mutateLocationURL(baseURL, func(parsed *url.URL) error {
			cleaned := strings.TrimSpace(value)
			parsed.Path = path.Clean("/" + strings.TrimPrefix(cleaned, "/"))
			return nil
		})
	case "search":
		return mutateLocationURL(baseURL, func(parsed *url.URL) error {
			parsed.RawQuery = strings.TrimLeft(strings.TrimSpace(value), "?")
			return nil
		})
	case "protocol":
		return mutateLocationURL(baseURL, func(parsed *url.URL) error {
			scheme := strings.TrimSpace(strings.TrimSuffix(value, ":"))
			if scheme == "" {
				return fmt.Errorf("location.protocol requires a non-empty value")
			}
			parsed.Scheme = scheme
			return nil
		})
	case "host":
		return mutateLocationURL(baseURL, func(parsed *url.URL) error {
			host := strings.TrimSpace(value)
			if host == "" {
				return fmt.Errorf("location.host requires a non-empty value")
			}
			parsed.Host = host
			return nil
		})
	case "hostname":
		return mutateLocationURL(baseURL, func(parsed *url.URL) error {
			host := strings.TrimSpace(value)
			if host == "" {
				return fmt.Errorf("location.hostname requires a non-empty value")
			}
			if port := parsed.Port(); port != "" {
				parsed.Host = net.JoinHostPort(host, port)
			} else {
				parsed.Host = host
			}
			return nil
		})
	case "port":
		return mutateLocationURL(baseURL, func(parsed *url.URL) error {
			trimmed := strings.TrimSpace(value)
			if trimmed == "" {
				if hostname := parsed.Hostname(); hostname != "" {
					parsed.Host = hostname
					return nil
				}
				return fmt.Errorf("location.port requires a current hostname")
			}
			if _, err := strconv.Atoi(trimmed); err != nil {
				return fmt.Errorf("unsupported location.port value: %s", value)
			}
			hostname := parsed.Hostname()
			if hostname == "" {
				return fmt.Errorf("location.port requires a current hostname")
			}
			parsed.Host = net.JoinHostPort(hostname, trimmed)
			return nil
		})
	case "username":
		return mutateLocationURL(baseURL, func(parsed *url.URL) error {
			username := strings.TrimSpace(value)
			password, hasPassword := "", false
			if parsed.User != nil {
				password, hasPassword = parsed.User.Password()
			}
			if username == "" && !hasPassword && password == "" {
				parsed.User = nil
				return nil
			}
			if hasPassword {
				parsed.User = url.UserPassword(username, password)
			} else {
				parsed.User = url.User(username)
			}
			return nil
		})
	case "password":
		return mutateLocationURL(baseURL, func(parsed *url.URL) error {
			password := strings.TrimSpace(value)
			username := ""
			if parsed.User != nil {
				username = parsed.User.Username()
			}
			if username == "" && password == "" {
				parsed.User = nil
				return nil
			}
			parsed.User = url.UserPassword(username, password)
			return nil
		})
	case "origin":
		return "", fmt.Errorf("location.origin is read-only")
	default:
		return "", fmt.Errorf("unsupported location property %q", property)
	}
}

func mutateLocationURL(baseURL string, mutate func(*url.URL) error) (string, error) {
	baseURL = strings.TrimSpace(baseURL)
	if baseURL == "" {
		return "", fmt.Errorf("location update requires a current URL")
	}
	parsed, err := url.Parse(baseURL)
	if err != nil {
		return "", err
	}
	if err := mutate(parsed); err != nil {
		return "", err
	}
	return parsed.String(), nil
}

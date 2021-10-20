import {useState, useEffect, useRef, SyntheticEvent, MouseEventHandler} from "react";
import { useLocation } from "react-router-dom";

import { useAuth } from "../useAuth";
import { useUnauthenticatedRequest } from "../util";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faGithub,
  faGitlab,
  faGoogle,
  IconDefinition,
} from "@fortawesome/free-brands-svg-icons";
import { faSignInAlt } from "@fortawesome/free-solid-svg-icons";

interface OAuthProviders {
  providers: string[];
}

export default function Login() {
  const location = useLocation();
  const auth = useAuth();

  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState<string | null>(null);
  const isMountedRef = useRef(null);

  const { response: oauthProviders } =
    useUnauthenticatedRequest<OAuthProviders>({
      endpoint: "auth/login/oauth/providers",
    });

  useEffect(() => {
    if (location.state?.error) {
      setError(location.state.error);
    }

    isMountedRef.current = true;
    return () => { isMountedRef.current = false };
  });

  const handleSubmit = async (evt: SyntheticEvent) => {
    evt.preventDefault();

    setError("");
    setLoading("password");

    try {
      await auth?.login(username, password);
    } catch (e: any) {
      setError(e.message);
    } finally {
      if (isMountedRef.current) {
        setLoading(null);
      }
    }
  };

  const handleOAuthLogin = async (provider: string) => {
    setError("");
    setLoading(provider);

    try {
      await auth?.oauthLogin(provider);
    } catch (e: any) {
      setError(e.message);
    }
  };

  return (
    <div className="p-4 min-vh-100 d-flex justify-content-center align-items-center">
      <div>
        <h1>chartered ✈️</h1>
        <h6>a private, authenticated cargo registry</h6>

        <div
          className="card border-0 shadow-sm text-black p-2"
          style={{ width: "40rem" }}
        >
          <div className="card-body">
            <div
              className="alert alert-danger alert-dismissible"
              role="alert"
              style={{ display: error ? "block" : "none" }}
            >
              {error}

              <button
                type="button"
                className="btn-close"
                aria-label="Close"
                onClick={() => setError("")}
              />
            </div>

            <form onSubmit={handleSubmit}>
              <div className="form-floating">
                <input
                  type="text"
                  className="form-control"
                  placeholder="john.smith"
                  id="username"
                  disabled={!!loading}
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                />

                <label htmlFor="email" className="form-label">
                  Username
                </label>
              </div>

              <div className="form-floating mt-2">
                <input
                  type="password"
                  className="form-control"
                  placeholder="&bull;&bull;&bull;&bull;&bull;&bull;&bull;&bull;&bull;&bull;&bull;&bull;"
                  id="password"
                  disabled={!!loading}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />

                <label htmlFor="password" className="form-label">
                  Password
                </label>
              </div>

              <ButtonOrSpinner
                type="submit"
                variant="primary"
                disabled={!!loading}
                showSpinner={loading === "password"}
                text={`Login`}
                icon={faSignInAlt}
                onClick={handleSubmit}
              />
            </form>

            {oauthProviders?.providers.length > 0 ? (
              <>
                <div className="side-lines mt-3">or</div>

                {oauthProviders.providers.map((v, i) => (
                  <ButtonOrSpinner
                    key={i}
                    type="button"
                    variant="dark"
                    disabled={!!loading}
                    showSpinner={loading === v}
                    text={`Login with ${
                      v.charAt(0).toUpperCase() + v.slice(1)
                    }`}
                    icon={getIconForProvider(v)[0]}
                    background={getIconForProvider(v)[1]}
                    onClick={(evt) => {
                      evt.preventDefault();
                      handleOAuthLogin(v);
                    }}
                  />
                ))}
              </>
            ) : (
              <></>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

const BRANDS = {
  default: [faSignInAlt, ""],
  github: [faGithub, "#4078c0"],
  gitlab: [faGitlab, "#6E49cb"],
  google: [faGoogle, "#4285f4"],
};

function getIconForProvider(provider: string): [IconDefinition, string] {
  return BRANDS[provider] || BRANDS.default;
}

function ButtonOrSpinner({
  type,
  variant,
  disabled,
  showSpinner,
  text,
  icon,
  background,
  onClick,
}: {
  type: "button" | "submit";
  variant: string;
  disabled: boolean;
  showSpinner: boolean;
  text: string;
  icon?: IconDefinition;
  background?: string;
  onClick: MouseEventHandler<HTMLButtonElement>;
}) {
  if (showSpinner) {
    return (
      <div
        className="spinner-border text-primary mt-3 m-auto d-block"
        role="status"
      >
        <span className="visually-hidden">Logging in...</span>
      </div>
    );
  }

  if (type) {
    return (
      <button
        type={type}
        disabled={disabled}
        onClick={onClick}
        className={`btn btn-lg mt-2 btn-${variant} w-100`}
        style={{ background, borderColor: background }}
      >
        {icon ? <FontAwesomeIcon icon={icon} className="me-2" /> : <></>}
        {text}
      </button>
    );
  }

  return <></>;
}

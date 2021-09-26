import React = require("react");
import { useState, useEffect } from "react";
import { Link, useHistory } from "react-router-dom";

import Nav from "../../sections/Nav";
import { useAuth } from "../../useAuth";
import { authenticatedEndpoint } from "../../util";

import { Plus } from "react-bootstrap-icons";

export default function CreateOrganisation() {
  const auth = useAuth();
  const router = useHistory();

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");

  const createOrganisation = async (evt) => {
    evt.preventDefault();

    setError("");
    setLoading(true);

    try {
      let res = await fetch(authenticatedEndpoint(auth, "organisations"), {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ name, description }),
      });
      let json = await res.json();

      if (json.error) {
        throw new Error(json.error);
      }

      setName("");
      setDescription("");
      router.push(`/crates/${name}`);
    } catch (e) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="text-white">
      <Nav />

      <div className="container mt-4 pb-4">
        <h1>Create New Organisation</h1>

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
          ></button>
        </div>

        <div className="card border-0 shadow-sm text-black">
          <div className="card-body">
            <form onSubmit={createOrganisation}>
              <div className="mb-3">
                <label htmlFor="org-name" className="form-label">
                  Name
                </label>
                <input
                  id="org-name"
                  type="text"
                  className="form-control"
                  pattern="[a-zA-Z0-9-]*"
                  placeholder="backend-team"
                  onChange={(e) => setName(e.target.value)}
                  disabled={loading}
                  value={name}
                />
                <div className="form-text">
                  Must be in the format <code>[a-zA-Z0-9-]*</code>
                </div>
              </div>

              <div>
                <label htmlFor="org-description" className="form-label">
                  Description
                </label>
                <textarea
                  id="org-description"
                  className="form-control"
                  rows={3}
                  onChange={(e) => setDescription(e.target.value)}
                  disabled={loading}
                  value={description}
                />
              </div>

              <div className="clearfix"></div>

              <button
                type="submit"
                className="btn btn-success mt-2 float-end"
                style={{ display: !loading ? "block" : "none" }}
              >
                Create
              </button>
              <div
                className="spinner-border text-primary mt-4 float-end"
                role="status"
                style={{ display: loading ? "block" : "none" }}
              >
                <span className="visually-hidden">Submitting...</span>
              </div>

              <Link
                to="/ssh-keys/list"
                className="btn btn-danger mt-2 float-end me-1"
              >
                Cancel
              </Link>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
}
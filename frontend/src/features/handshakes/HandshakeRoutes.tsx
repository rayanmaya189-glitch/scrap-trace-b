import React from 'react';
import { Routes, Route } from 'react-router-dom';

export function HandshakeRoutes() {
  return (
    <Routes>
      <Route index element={<div className="p-4">Handshake List (Coming Soon)</div>} />
      <Route path="initiate" element={<div className="p-4">Initiate Handshake (Coming Soon)</div>} />
      <Route path=":id" element={<div className="p-4">Handshake Detail (Coming Soon)</div>} />
    </Routes>
  );
}

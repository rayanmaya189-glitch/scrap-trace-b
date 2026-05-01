import React from 'react';
import { Routes, Route } from 'react-router-dom';
import { HandshakeInitiator } from './HandshakeInitiator';

export function HandshakeRoutes() {
  return (
    <Routes>
      <Route path="/" element={<HandshakeInitiator />} />
      <Route path="/initiate" element={<HandshakeInitiator />} />
    </Routes>
  );
}

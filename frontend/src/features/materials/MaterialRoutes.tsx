import React from 'react';
import { Routes, Route } from 'react-router-dom';
import { MaterialList } from './MaterialList';
import { CreateMaterialForm } from './CreateMaterialForm';

export function MaterialRoutes() {
  return (
    <Routes>
      <Route index element={<MaterialList />} />
      <Route path="new" element={<CreateMaterialForm />} />
      <Route path=":id" element={<div className="p-4">Material Detail (Coming Soon)</div>} />
    </Routes>
  );
}
